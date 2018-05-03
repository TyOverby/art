use super::Expr;
use rand::Rng;

fn get_rand<'a, T, R: Rng>(weighted: &'a [(f32, T)], rng: &mut R) -> &'a T {
    if weighted.is_empty() {
        panic!("no");
    }
    let total: f32 = weighted.iter().map(|a| a.0).sum();
    let rand = rng.gen::<f32>() * total;

    let mut counter = 0.0;
    for (w, v) in weighted {
        counter += w;
        if rand < counter {
            return v;
        }
    }
    return &weighted.iter().last().as_ref().unwrap().1;
}

pub(crate) fn get<'b, R: Rng>(rng: &'b mut R) -> Expr {
    return get_rand(
        &[
            (
                3.0,
                Box::new(|_: &'b mut _| Expr::A) as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                3.0,
                Box::new(|_: &'b mut _| Expr::B) as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                1.0,
                Box::new(|rng: &'b mut _| Expr::Or(Box::new(get(rng)), Box::new(get(rng))))
                    as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                1.0,
                Box::new(|rng: &'b mut _| Expr::And(Box::new(get(rng)), Box::new(get(rng))))
                    as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                1.0,
                Box::new(|rng: &'b mut _| Expr::Xor(Box::new(get(rng)), Box::new(get(rng))))
                    as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                1.0,
                Box::new(|rng: &'b mut _| Expr::Add(Box::new(get(rng)), Box::new(get(rng))))
                    as Box<Fn(&'b mut R) -> Expr>,
            ),
            (
                1.0,
                Box::new(|rng: &'b mut _| Expr::Sub(Box::new(get(rng)), Box::new(get(rng))))
                    as Box<Fn(&'b mut R) -> Expr>,
            ),
        ],
        rng,
    )(rng);
}
