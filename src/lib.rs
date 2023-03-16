use bevy::utils::hashbrown::hash_set::HashSet;
use core::hash::{Hash, Hasher};
use ordered_float::OrderedFloat;
use robust::{insphere, Coord3D};

// Returns a set of edges representing the 3d delauney triangulation of the passed in points
pub fn tetrahedralize(vertices: &Vec<Vertex>) -> Option<HashSet<Edge>> {
    if vertices.is_empty() {
        // nothing to tetrahedralize
        return None;
    }

    // construct super tetrahedron (analagous to super triangle in 2d algorithm) that encapsulates
    // all given points
    let st = make_super_tetrahedron(vertices);

    let mut tetrahedrons: Vec<Tetrahedron> = Vec::new();
    tetrahedrons.push(st);

    for vertex in vertices {
        let mut triangles = Vec::new();

        for mut t in &mut tetrahedrons {
            if Tetrahedron::in_circumsphere(*t, vertex) {
                t.is_bad = true;
                triangles.push(Triangle::new(t.a, t.b, t.c));
                triangles.push(Triangle::new(t.a, t.b, t.d));
                triangles.push(Triangle::new(t.a, t.c, t.d));
                triangles.push(Triangle::new(t.b, t.c, t.d));
            }

            // remove duplicate triangles
            for i in 0..triangles.len() {
                for j in i..triangles.len() {
                    if triangles[i].almost_equal(&triangles[j]) {
                        triangles[i].is_bad = true;
                        triangles[j].is_bad = true;
                    }
                }
            }
        }

        tetrahedrons = tetrahedrons
            .iter()
            .copied()
            // .map(|t| *t)
            .filter(|t| t.is_bad)
            .collect();
        triangles = triangles
            .iter()
            .copied()
            // .map(|t| *t)
            .filter(|t| t.is_bad)
            .collect();

        // create new tetrahedrons from unique triangles and new vertex
        for t in triangles {
            tetrahedrons.push(Tetrahedron::new(t.a, t.b, t.c, *vertex))
        }
    }

    // remove all tetrahedrons containing a vertex in the super tetrahedron since it wasn't part
    // of the original tetrahedralization
    // TODO: rename
    tetrahedrons = tetrahedrons
        .iter()
        .copied()
        // .map(|t| *t)
        .filter(|t| {
            !(t.contains_vertex(&st.a)
                || t.contains_vertex(&st.b)
                || t.contains_vertex(&st.c)
                || t.contains_vertex(&st.d))
        })
        .collect();

    let mut edges = HashSet::new();
    for t in tetrahedrons {
        edges.insert(Edge::new(t.a, t.b));
        edges.insert(Edge::new(t.b, t.c));
        edges.insert(Edge::new(t.c, t.a));
        edges.insert(Edge::new(t.d, t.a));
        edges.insert(Edge::new(t.d, t.b));
        edges.insert(Edge::new(t.d, t.c));
    }

    Some(edges)
}

fn make_super_tetrahedron(vertices: &[Vertex]) -> Tetrahedron {
    let mut x_min = vertices[0].coord.x;
    let mut y_min = vertices[0].coord.y;
    let mut z_min = vertices[0].coord.z;
    let mut x_max = x_min;
    let mut y_max = y_min;
    let mut z_max = z_min;

    for v in vertices.iter().skip(1) {
        let px = v.coord.x;
        let py = v.coord.y;
        let pz = v.coord.z;

        if px < x_min {
            x_min = px;
        } else if px > x_max {
            x_max = px;
        }

        if py < y_min {
            y_min = py;
        } else if py > y_max {
            y_max = py;
        }

        if pz < z_min {
            z_min = pz;
        } else if pz > z_max {
            z_max = pz;
        }
    }

    let dx = x_max - x_min;
    let dy = y_max - y_min;
    let dz = z_max - z_min;
    let d_max = dx.max(dy.max(dz)) * 2.;

    Tetrahedron::new(
        Vertex {
            coord: Coord3D::<OrderedFloat<f64>> {
                x: x_min - 1.,
                y: y_min - 1.,
                z: z_min - 1.,
            },
        },
        Vertex {
            coord: Coord3D::<OrderedFloat<f64>> {
                x: x_max + d_max,
                y: y_min - 1.,
                z: z_min - 1.,
            },
        },
        Vertex {
            coord: Coord3D::<OrderedFloat<f64>> {
                x: x_min - 1.,
                y: y_max + d_max,
                z: z_min - 1.,
            },
        },
        Vertex {
            coord: Coord3D::<OrderedFloat<f64>> {
                x: x_min - 1.,
                y: y_min - 1.,
                z: z_max + d_max,
            },
        },
    )
}

#[derive(Copy, Clone)]
struct Tetrahedron {
    // tetrahedron vertices
    a: Vertex,
    b: Vertex,
    c: Vertex,
    d: Vertex,
    // marker for incremental invalidation
    is_bad: bool,
}

impl Tetrahedron {
    fn new(a: Vertex, b: Vertex, c: Vertex, d: Vertex) -> Tetrahedron {
        Tetrahedron {
            a,
            b,
            c,
            d,
            is_bad: false,
        }
    }

    // returns whether point is inside the circumsphere constructed via the vertices
    // of the tetrahedron
    fn in_circumsphere(t: Tetrahedron, v: &Vertex) -> bool {
        insphere(t.a.coord, t.b.coord, t.c.coord, t.d.coord, v.coord) > 0.
    }

    fn contains_vertex(&self, v: &Vertex) -> bool {
        v.almost_equal(&self.a)
            || v.almost_equal(&self.b)
            || v.almost_equal(&self.c)
            || v.almost_equal(&self.d)
    }
}

#[derive(Copy, Clone)]
struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
    // marker for incremental invalidation
    is_bad: bool,
}

impl Triangle {
    fn new(a: Vertex, b: Vertex, c: Vertex) -> Triangle {
        Triangle {
            a,
            b,
            c,
            is_bad: false,
        }
    }

    fn almost_equal(&self, triangle: &Triangle) -> bool {
        (self.a.almost_equal(&triangle.a)
            || self.a.almost_equal(&triangle.b)
            || self.a.almost_equal(&triangle.c))
            && (self.b.almost_equal(&triangle.a)
                || self.b.almost_equal(&triangle.b)
                || self.b.almost_equal(&triangle.c))
            && (self.c.almost_equal(&triangle.a)
                || self.c.almost_equal(&triangle.b)
                || self.c.almost_equal(&triangle.c))
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct Edge {
    pub a: Vertex,
    pub b: Vertex,
}

impl Edge {
    fn new(a: Vertex, b: Vertex) -> Edge {
        Edge { a, b }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub coord: Coord3D<OrderedFloat<f64>>,
}

impl Vertex {
    pub fn new(x: f64, y: f64, z: f64) -> Vertex {
        Vertex {
            coord: Coord3D {
                x: OrderedFloat(x),
                y: OrderedFloat(y),
                z: OrderedFloat(z),
            },
        }
    }

    fn almost_equal(&self, v: &Vertex) -> bool {
        (self.coord.x - v.coord.x).powf(2.)
            + (self.coord.y - v.coord.y).powf(2.)
            + (self.coord.z - v.coord.z).powf(2.)
            < 0.01
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Vertex) -> bool {
        self.coord.x.eq(&other.coord.x)
            && self.coord.y.eq(&other.coord.y)
            && self.coord.z.eq(&other.coord.z)
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.coord.x.hash(state);
        self.coord.y.hash(state);
        self.coord.z.hash(state);
    }
}
