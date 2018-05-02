#![allow(dead_code)]

extern crate cretonne;
extern crate cretonne_codegen;
extern crate cretonne_module;
extern crate cretonne_simplejit;
extern crate rand;
extern crate vectorphile;

mod generate;

use cretonne::prelude::*;
use cretonne_codegen::ir::types::I32;
use cretonne_module::{Linkage, Module};
use cretonne_simplejit::{SimpleJITBackend, SimpleJITBuilder};
use rand::{random, Rng, SeedableRng, XorShiftRng};
use std::fs::File;
use std::io::BufWriter;
use vectorphile::svg::SvgBackend;
use vectorphile::Canvas;

#[derive(Eq, PartialEq)]
enum Expr {
    Zero,
    X,
    Y,
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn info(&self) -> (bool, bool) {
        use Expr::*;
        fn or((la, ra): (bool, bool), (lb, rb): (bool, bool)) -> (bool, bool) {
            (la | lb, ra | rb)
        }

        match self {
            &Zero => (false, false),
            &X => (true, false),
            &Y => (false, true),
            &Or(ref a, ref b) => or(a.info(), b.info()),
            &And(ref a, ref b) => or(a.info(), b.info()),
            &Xor(ref a, ref b) => or(a.info(), b.info()),
            &Add(ref a, ref b) => or(a.info(), b.info()),
            &Sub(ref a, ref b) => or(a.info(), b.info()),
        }
    }

    fn simplify(self) -> Self {
        use Expr::*;
        match self {
            Zero => Zero,
            X => X,
            Y => Y,
            And(a, b) => match (a.simplify(), b.simplify()) {
                (Zero, _) => Zero,
                (_, Zero) => Zero,
                (a, b) => And(Box::new(a), Box::new(b)),
            },
            Or(a, b) => match (a.simplify(), b.simplify()) {
                (Zero, b) => b,
                (a, Zero) => a,
                (a, b) => {
                    if a == b {
                        a
                    } else {
                        Or(Box::new(a), Box::new(b))
                    }
                }
            },
            Xor(a, b) => {
                let (a, b) = (a.simplify(), b.simplify());
                if a == b {
                    Zero
                } else {
                    Xor(Box::new(a), Box::new(b))
                }
            }
            Sub(a, b) => {
                let (a, b) = (a.simplify(), b.simplify());
                if a == b {
                    Zero
                } else {
                    Sub(Box::new(a), Box::new(b))
                }
            }
            add @ Add(_, _) => add,
        }
    }
}

fn compile(expr: Expr) -> (fn(u32, u32) -> u32) {
    let builder = SimpleJITBuilder::new();
    let mut module: Module<SimpleJITBackend> = Module::new(builder);
    let mut function_builder_context = FunctionBuilderContext::<Variable>::new();
    let mut ctx = module.make_context();

    ctx.func.signature.params.push(AbiParam::new(I32));
    ctx.func.signature.params.push(AbiParam::new(I32));
    ctx.func.signature.returns.push(AbiParam::new(I32));

    {
        let mut builder =
            FunctionBuilder::<Variable>::new(&mut ctx.func, &mut function_builder_context);
        let entry_ebb = builder.create_ebb();
        builder.append_ebb_params_for_function_params(entry_ebb);
        builder.switch_to_block(entry_ebb);
        builder.seal_block(entry_ebb);

        let (x, y) = declare_variables(I32, &mut builder, entry_ebb);
        let u255 = builder.ins().iconst(I32, 255);
        let result = compile_expr(expr, x, y, u255, &mut builder);
        builder.ins().return_(&[result]);
        builder.finalize();
    }

    let id = module
        .declare_function("jitted", Linkage::Export, &ctx.func.signature)
        .unwrap();
    module.define_function(id, &mut ctx).unwrap();
    module.clear_context(&mut ctx);
    let code = module.finalize_function(id);
    return unsafe { ::std::mem::transmute(code) };
}

fn compile_expr(
    expr: Expr,
    x: Variable,
    y: Variable,
    u255: Value,
    builder: &mut FunctionBuilder<Variable>,
) -> Value {
    match expr {
        Expr::X => builder.use_var(x),
        Expr::Y => builder.use_var(y),
        Expr::And(l, r) => {
            let l = compile_expr(*l, x, y, u255, builder);
            let r = compile_expr(*r, x, y, u255, builder);
            builder.ins().band(l, r)
        }
        Expr::Or(l, r) => {
            let l = compile_expr(*l, x, y, u255, builder);
            let r = compile_expr(*r, x, y, u255, builder);
            builder.ins().bor(l, r)
        }
        Expr::Xor(l, r) => {
            let l = compile_expr(*l, x, y, u255, builder);
            let r = compile_expr(*r, x, y, u255, builder);
            builder.ins().bxor(l, r)
        }
        Expr::Add(l, r) => {
            let l = compile_expr(*l, x, y, u255, builder);
            let r = compile_expr(*r, x, y, u255, builder);
            let i = builder.ins().iadd(l, r);
            builder.ins().urem(i, u255)
        }
        Expr::Sub(l, r) => {
            let l = compile_expr(*l, x, y, u255, builder);
            let r = compile_expr(*r, x, y, u255, builder);
            let i = builder.ins().isub(l, r);
            builder.ins().urem(i, u255)
        }
        Expr::Zero => builder.ins().iconst(I32, 0),
    }
}

fn declare_variables(
    int: types::Type,
    builder: &mut FunctionBuilder<Variable>,
    entry_ebb: Ebb,
) -> (Variable, Variable) {
    let val_x = builder.ebb_params(entry_ebb)[0];
    let var_x = Variable::new(0);
    builder.declare_var(var_x, int);
    builder.def_var(var_x, val_x);

    let val_y = builder.ebb_params(entry_ebb)[1];
    let var_y = Variable::new(1);
    builder.declare_var(var_y, int);
    builder.def_var(var_y, val_y);
    (var_x, var_y)
}

#[test]
fn test_x() {
    assert_eq!(compile(Expr::X)(5, 6), 5);
}

#[test]
fn test_y() {
    assert_eq!(compile(Expr::Y)(5, 6), 6);
}

#[test]
fn test_and() {
    assert_eq!(
        compile(Expr::And(Box::new(Expr::X), Box::new(Expr::Y)))(5, 4),
        5 & 4
    );
    assert_eq!(
        compile(Expr::And(Box::new(Expr::X), Box::new(Expr::Y)))(999, 2 * 2 * 2),
        999 & (2 * 2 * 2)
    );
}

#[test]
fn test_or() {
    assert_eq!(
        compile(Expr::Or(Box::new(Expr::X), Box::new(Expr::Y)))(5, 4),
        5 | 4
    );
    assert_eq!(
        compile(Expr::Or(Box::new(Expr::X), Box::new(Expr::Y)))(999, 2 * 2 * 2),
        999 | (2 * 2 * 2)
    );
}

#[test]
fn test_xor() {
    assert_eq!(
        compile(Expr::Xor(Box::new(Expr::X), Box::new(Expr::Y)))(5, 4),
        5 ^ 4
    );
    assert_eq!(
        compile(Expr::Xor(Box::new(Expr::X), Box::new(Expr::Y)))(999, 2 * 2 * 2),
        999 ^ (2 * 2 * 2)
    );
}

#[test]
fn test_sub() {
    assert_eq!(
        compile(Expr::Sub(Box::new(Expr::X), Box::new(Expr::Y)))(5, 4),
        5 - 4
    );
    assert_eq!(
        compile(Expr::Sub(Box::new(Expr::X), Box::new(Expr::Y)))(999, 2 * 2 * 2),
        (999 - (2 * 2 * 2)) % 255
    );
}

#[test]
fn test_add() {
    assert_eq!(
        compile(Expr::Add(Box::new(Expr::X), Box::new(Expr::Y)))(5, 4),
        5 + 4
    );
    assert_eq!(
        compile(Expr::Add(Box::new(Expr::X), Box::new(Expr::Y)))(999, 2 * 2 * 2),
        (999 + (2 * 2 * 2)) % 255
    );
}

fn get_until_ok<R: Rng>(rng: &mut R) -> (fn(u32, u32) -> u32) {
    loop {
        let attempt = generate::get(rng);
        //let (l, r) = attempt.info();
        // if l && r {
        return compile(attempt);
        //} else {
        //    println!("skipping");
        //}
    }
}

fn main() {
    let rand_arr: [u32; 4] = [random(), random(), random(), random()];
    let filename = format!(
        "./out/{}_{}_{}_{}.svg",
        rand_arr[0], rand_arr[1], rand_arr[2], rand_arr[3]
    );
    let mut rng = XorShiftRng::from_seed(rand_arr);

    let x1p = get_until_ok(&mut rng);
    let x2p = get_until_ok(&mut rng);
    let y1p = get_until_ok(&mut rng);
    let y2p = get_until_ok(&mut rng);

    let file = BufWriter::new(File::create(filename).unwrap());
    let mut canvas = Canvas::new(SvgBackend::new_with_bb(file, 0.0, 0.0, 255.0, 255.0).unwrap());

    for _ in 0..255 {
        let x: u32 = rng.gen::<u32>() % 255;
        let y: u32 = rng.gen::<u32>() % 255;

        let x1 = x1p(x, y);
        let x2 = x2p(x, y);
        let y1 = y1p(x, y);
        let y2 = y2p(x, y);
        canvas.draw_line((x1, x2), (y1, y2), None).unwrap();
    }

    canvas.close().unwrap()
}
