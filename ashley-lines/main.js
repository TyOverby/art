/*window.setTimeout(() => {
    window.location = window.location;
}, 1000);*/

var numMoveTos = 0
var numLineTos = 0
const weird = 50;
const weird_balance = 10;

class Thing {
    constructor() {
        this.buffer = "";
        this.oldX = 0;
        this.oldY = 0;
    }

    moveTo(x, y) {
        numMoveTos++
        this.oldX = x;
        this.oldY = y;
        this.buffer += "M" + x + "," + y
    }

    lineTo(x, y) {
        const clamp = (low, t, high) => {
            if (t < low) return low;
            if (t > high) return high;
            return t;
        };

        numLineTos++
        let dist = Math.sqrt(Math.pow(x - this.oldX, 2) + Math.pow(y - this.oldY, 2));
        let w = Math.log(clamp(0, x - 250, 1000) / 1000 + 1.0) * (Math.sqrt(dist * weird) / 2);
        var offsetX1 = (Math.sign(Math.random() - 0.5) * w);
        var offsetY1 = (Math.sign(Math.random() - 0.5) * w);
        var offsetX2 = (Math.sign(Math.random() - 0.5) * w);
        var offsetY2 = (Math.sign(Math.random() - 0.5) * w);
        this.buffer += "C" + (x + offsetX2) + "," + (y + offsetY2) + "," + (x + offsetX1) + "," + (y + offsetY1) + "," + (x) + "," +(y)

    }

    finish() {
        console.log(numMoveTos)
        console.log(numLineTos)
        var result = "<svg xmlns=\"http://www.w3.org/2000/svg\">"
        result += "<path fill=\"none\" stroke=\"black\" d=\""
        result += this.buffer
        result += "\"></path>"
        result += "</svg>"
        return result
    }
}

class Point {
    constructor(x, y) {
        this.x = x
        this.y = y
    }

    sub(other) {
        return new Point(this.x - other.x, this.y - other.y)
    }

    len() {
        return Math.sqrt(this.x * this.x + this.y * this.y)
    }

    dist(other) {
        return this.sub(other).len()
    }

    eq(other) {
        return this.x === other.x && this.y === other.y
    }
}

class Line {
    constructor(start, end) {
        this.start = start
        this.end = end
    }

    swapStartEnd() {
        var tmp = this.end;
        this.end = this.start;
        this.start = tmp;
    }

    intersects(other) {
        if (this.start.eq(other.start) || this.start.eq(other.end)) { return null }
        if (this.end.eq(other.start) || this.end.eq(other.end)) { return null }
        var s1_x, s1_y, s2_x, s2_y;
        s1_x = this.end.x - this.start.x;
        s1_y = this.end.y - this.start.y;
        s2_x = other.end.x - other.start.x;
        s2_y = other.end.y - other.start.y;
        var s, t;
        s = (-s1_y * (this.start.x - other.start.x) + s1_x * (this.start.y - other.start.y)) / (-s2_x * s1_y + s1_x * s2_y);
        t = (s2_x * (this.start.y - other.start.y) - s2_y * (this.start.x - other.start.x)) / (-s2_x * s1_y + s1_x * s2_y);
        if (s >= 0 && s <= 1 && t >= 0 && t <= 1) {
            // Collision detected
            var intX = this.start.x + (t * s1_x);
            var intY = this.start.y + (t * s1_y);
            return new Point(intX, intY);
        }
        return null; // No collision
    }

    len() {
        return this.start.dist(this.end)
    }
}

var scale = 1000
var center = new Point(scale / 2, scale / 2)
function randPoint() {
    while (true) {
        var point = new Point(Math.random() * scale, Math.random() * scale)
        var blah = point.sub(center).len()
        if (blah < scale / 2) {
            return point
        }
    }
}
function randPointOnCircle() {
    
}

cloud = []
for (var index = 0; index < 500; index++) {
    cloud.push(randPoint())
}

lines = []
for (var index = 0; index < cloud.length; index++) {
    for (var otherIndex = index + 1; otherIndex < cloud.length; otherIndex++) {
        lines.push(new Line(cloud[index], cloud[otherIndex]))
    }
}

lines.sort(function (l, r) { return l.len() - r.len() })

lines = lines.filter(function (value, index) {
    var val = !lines.some(function (otherValue, otherIndex) {
        return otherIndex < index && value.intersects(otherValue) != null && otherValue.fuck
    })
    value.fuck = val
    return val;
})

function shuffleArray(array) {
    for (var i = array.length - 1; i > 0; i--) {
        var j = Math.floor(Math.random() * (i + 1));
        var temp = array[i];
        array[i] = array[j];
        array[j] = temp;
    }
}
shuffleArray(lines)

for (var index = 0; index < lines.length; index++) {
    lines[index].visited = false
}

function randomWalk(thing, current) {
    current.visited = true
    thing.lineTo(current.end.x, current.end.y)
    var found = lines.filter(function (value, index, array) {
        return value.visited === false && (value.start.eq(current.end) || value.end.eq(current.end))
    })
    for (var index = 0; index < found.length; index++) {
        var found1 = found[index]
        if (found1.visited) {
            continue
        }
        if (current.end.eq(found1.end)) {
            found1.swapStartEnd()
        }
        if (index != 0) {
            thing.moveTo(found1.start.x, found1.start.y)
        }
        randomWalk(thing, found1)
    }
}

var thing = new Thing();

thing.moveTo(lines[0].start.x, lines[0].start.y)
randomWalk(thing, lines[0])

var mainBody = document.getElementById("mainBody")
mainBody.innerHTML = thing.finish()
