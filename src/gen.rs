use crate::dsu::{ UnionFind, Unite };

#[derive(Debug, Copy, Clone)]
struct Dummy();

impl Unite for Dummy {
    fn unite(self, oth: Self, u: usize, v: usize) -> Option<Self> { Some(Dummy()) }
}

#[derive(Debug, Clone)]
pub struct GenFlow {
    heads: Vec<bool>,
    h: usize,
    w: usize,

    down: Vec<bool>,
    right: Vec<bool>,
    dsu: UnionFind<Dummy>,

    search_order: Vec<usize>,
    search_depth: usize,

    allow_zigzag: bool,
}

impl GenFlow {
    pub fn new(h: usize, w: usize, allow_zigzag: bool) -> Self {
        // let mut search_order: Vec<usize> = Vec::new();
        // for d in 0..(h + w - 1) {
        //     for r in 0..h {
        //         if let Some(c) = d.checked_sub(r) {
        //             if c < w {
        //                 search_order.push(r * w + c);
        //             }
        //         }
        //     }
        // }

        let search_order: Vec<usize> = (0..(h*w)).collect();

        return Self {
            h, w,
            heads: vec![false; h * w],
            dsu: UnionFind::from(vec![Dummy(); h * w]),
            down: vec![false; h * w],
            right: vec![false; h * w],
            search_order,
            search_depth: 0,
            allow_zigzag,
        };
    }

    pub fn depth(&self) -> usize { self.search_depth }

    pub fn len(&self) -> usize { self.h * self.w }

    pub fn solved(&self) -> bool { self.depth() == self.len() }

    fn get_degree(&self, u: usize) -> usize {
        ((u >= self.w && self.down[u - self.w]) as usize) +
        ((u % self.w > 0 && self.right[u - 1]) as usize)
    }

    pub fn extend(&mut self, head: bool, down: bool, right: bool) -> bool {
        let u = self.search_order[self.depth()];
        let degree = (head as usize) + (down as usize) + (right as usize) + self.get_degree(u);

        if degree != 2 ||
           down && u + self.w >= self.len() ||
           right && (u + 1) % self.w == 0 {
            return false;
        }

        if !self.allow_zigzag {
            if right && u >= self.w {
                let edges = (self.right[u - self.w] as usize) +
                    (self.down[u - self.w + 1] as usize) +
                    (self.down[u - self.w] as usize);
                    
                if edges >= 2 {
                    return false;
                }
            }
            if down && u % self.w > 0 && self.right[u - 1] && self.down[u - 1] {
                return false;
            }
        }

        if down {
            if !self.dsu.unite(u, u + self.w) {
                return false;
            }
            self.down[u] = true;
        }

        if right {
            if !self.dsu.unite(u, u + 1) {
                if down {
                    self.dsu.undo();
                    self.down[u] = false;
                }
                return false;
            }
            self.right[u] = true;
        }

        self.heads[u] = head;
        self.search_depth += 1;

        return true;
    }

    pub fn undo(&mut self) {
        self.search_depth -= 1;

        let u = self.search_order[self.depth()];

        if self.right[u] {
            self.dsu.undo();
            self.right[u] = false;
        }

        if self.down[u] {
            self.dsu.undo();
            self.down[u] = false;
        }

        self.heads[u] = false;
    }

    pub fn get_board(&self) -> Vec<Vec<usize>> {
        let mut heads: Vec<usize> = Vec::new();
        let mut board: Vec<Vec<usize>> = (0..self.h).map(|_| vec![0; self.w]).collect();

        for u in 0..self.len() {
            if self.heads[u] {
                let mut matched = false;
                for (i, &v) in heads.iter().enumerate() {
                    if self.dsu.same(u, v) {
                        board[u / self.w][u % self.w] = i + 1;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    heads.push(u);
                    board[u / self.w][u % self.w] = heads.len();
                }
            }
        }
        
        return board;
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        let mut row = String::new();
        for u in 0..self.len() {
            if u % self.w == 0 && u > 0 {
                result.push_str(&format!("\n{}\n", row));
                row = String::new();
            }

            let color = 0;

            result.push_str(&format!("{:02}", color));
            result.push(if self.right[u] { '-' } else { ' ' });

            row.push_str(if self.down[u] { " | " } else { "   " });
        }
        return result;
    }
}
