use crate::dsu::UnionFind;
use link::Link;
use modnum::Modnum;

mod link;
mod modnum;

const HASH_BASE: Modnum = Modnum(10_007);

#[derive(Debug, Clone, Copy)]
pub struct SearchConfig {
    pub allow_zigzag: bool,
    pub use_vcut: bool,
    pub use_diagonals: bool,
}

#[derive(Debug, Clone)]
pub struct SearchFlow {
    heads: Vec<Option<usize>>,
    h: usize,
    w: usize,

    down: Vec<bool>,
    right: Vec<bool>,
    dsu: UnionFind<Link>,

    search_order: Vec<usize>,
    search_depth: usize,

    diagonal_head_count: [Vec<usize>; 2],

    // TODO: move this out of struct
    hash_multiplier: Vec<Modnum>,

    config: SearchConfig,
}

impl SearchFlow {
    pub fn from_with_config(board: &Vec<Vec<usize>>, config: SearchConfig) -> Self {
        let (h, w) = (board.len(), board[0].len());

        let heads: Vec<Option<usize>> = board.iter().flatten()
            .map(|&cell| if cell == 0 { None } else { Some(cell) })
            .collect();

        let dsu = UnionFind::from(
            heads.iter().enumerate().map(|(i, cell)| {
                if let Some(color) = cell {
                    Link::Colored(*color, i)
                } else {
                    Link::Uncolored(i, i)
                }
            }).collect()
        );

        let mut search_order: Vec<usize> = Vec::new();
        for d in 0..(h + w - 1) {
            for r in 0..h {
                if let Some(c) = d.checked_sub(r) {
                    if c < w {
                        search_order.push(r * w + c);
                    }
                }
            }
        }

        let mut diagonal_head_count = [vec![0; h + w], vec![0; h + w]];
        for r in 0..h {
            for c in 0..w {
                if heads[r * w + c].is_some() {
                    diagonal_head_count[0][r + c] += 1;
                    diagonal_head_count[1][r + w - c] += 1;
                }
            }
        }

        let mut hash_multiplier: Vec<Modnum> = vec![Modnum(0); 2 * h * w];
        hash_multiplier[0] = Modnum(1);
        for i in 1..(2 * h * w) {
            hash_multiplier[i] = hash_multiplier[i - 1] * HASH_BASE;
        }

        return Self {
            h, w, heads, dsu,
            down: vec![false; h * w],
            right: vec![false; h * w],
            search_order,
            search_depth: 0,
            diagonal_head_count,
            hash_multiplier,
            config,
        };
    }

    pub fn depth(&self) -> usize { self.search_depth }

    pub fn len(&self) -> usize { self.h * self.w }

    pub fn solved(&self) -> bool { self.depth() == self.len() }

    fn next_depth(&mut self) { self.search_depth += 1; }

    fn prev_depth(&mut self) { self.search_depth -= 1; }

    fn is_head(&self, u: usize) -> bool { self.heads[u].is_some() }

    fn get_degree(&self, u: usize) -> usize {
        ((u >= self.w && self.down[u - self.w]) as usize) +
        ((u % self.w > 0 && self.right[u - 1]) as usize)
    }

    pub fn extend(&mut self, down: bool, right: bool) -> bool {
        assert!(!self.solved());

        let u = self.search_order[self.depth()];

        let up = u >= self.w && self.down[u - self.w];
        let left = u % self.w > 0 && self.right[u - 1];

        let degree = (down as usize) + (right as usize) + (up as usize) + (left as usize) +
            (self.is_head(u) as usize);

        if degree != 2 ||
           down && u + self.w >= self.len() ||
           right && (u + 1) % self.w == 0 {
            return false;
        }

        if !self.config.allow_zigzag {
            if right && up && self.right[u - self.w] ||
                right && up && self.down[u - self.w + 1] ||
                right && u >= self.w && self.down[u - self.w + 1] && self.right[u - self.w] ||
                down && left && self.down[u - 1] {
                return false;
            }

            if self.config.use_diagonals {
                if down && left && self.diagonal_head_count[0][u / self.w + u % self.w] == 0 ||
                    down && right && self.diagonal_head_count[1][u / self.w + self.w - u % self.w] == 0 {
                    return false;
                }
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

        if self.heads[u].is_some() {
            self.diagonal_head_count[0][u / self.w + u % self.w] -= 1;
            self.diagonal_head_count[1][u / self.w + self.w - u % self.w] -= 1;
        }

        self.next_depth();

        return true;
    }

    pub fn undo(&mut self) {
        assert_ne!(self.depth(), 0);

        self.prev_depth();

        let u = self.search_order[self.depth()];

        if self.heads[u].is_some() {
            self.diagonal_head_count[0][u / self.w + u % self.w] += 1;
            self.diagonal_head_count[1][u / self.w + self.w - u % self.w] += 1;
        }

        if self.right[u] {
            self.dsu.undo();
            self.right[u] = false;
        }

        if self.down[u] {
            self.dsu.undo();
            self.down[u] = false;
        }
    }

    pub fn get_state(&self) -> Vec<usize> {
        (0..self.len()).map(|u| {
            match self.dsu.get_data(u) {
                Link::Colored(color, s) => {
                    if u == s { color } else { 0 }
                },
                Link::Uncolored(s, t) => {
                    if u == s {
                        self.len() + t
                    } else if u == t {
                        self.len() + s
                    } else {
                        0
                    }
                },
                Link::Complete(_) => 0,
            }
        }).collect()
    }

    fn vcut(&self) -> bool {
        // let u = self.search_depth;
        // let mut colored_heads: HashMap<usize, usize> = HashMap::new();
        // let mut uncolored_stack: Vec<(usize, usize)> = Vec::new();
        // let mut load = vec![0; self.w];

        // for c in 0..self.w {
        //     let v = (u - u % self.w) + c + (if c < u % self.w { self.w } else { 0 });
        //     if v >= self.len() {
        //         continue;
        //     }

        //     let degree = ((v >= self.w && self.down[v - self.w]) as usize) +
        //         ((v % self.w > 0 && self.right[v - 1]) as usize);
            
        //     if let Some(color) = self.heads[v] {
        //         if degree == 0 {
        //             if colored_heads.contains_key(&color) {
        //                 load[colored_heads[&color] + 1] += 1;
        //                 load[c] -= 1;
        //             } else {
        //                 colored_heads.insert(color, c);
        //             }
        //         }
        //     } else {
        //         if degree == 1 {
        //             match self.dsu.get_color(v) {
        //                 Some(color) => {
        //                     if colored_heads.contains_key(&color) {
        //                         load[colored_heads[&color] + 1] += 1;
        //                         load[c] -= 1;
        //                     } else {
        //                         colored_heads.insert(color, c);
        //                     }
        //                 },
        //                 None => {
        //                     if !uncolored_stack.is_empty() &&
        //                             self.dsu.same(v, uncolored_stack.last().unwrap().0) {
        //                         load[uncolored_stack.pop().unwrap().1 + 1] -= 1;
        //                         load[c] += 1;
        //                     } else {
        //                         uncolored_stack.push((v, c));
        //                     }
        //                 },
        //             }
        //         } else if degree == 0 {
        //             load[c] -= 1;
        //             if c + 1 < self.w {
        //                 load[c + 1] += 1;
        //             }
        //         }
        //     }
        // }

        // for c in 1..self.w {
        //     load[c] += load[c - 1];
        // }

        // for c in 0..self.w {
        //     let mut v = (u - u % self.w) + c + (if c < u % self.w { 2 * self.w } else { self.w });
        //     while load[c] > 0 && v < self.len() {
        //         if self.heads[v].is_none() {
        //             load[c] -= 1;
        //         }
        //         v += self.w;
        //     }
        //     if load[c] > 0 {
        //         return true;
        //     }
        // }

        return false;
    }

    pub fn feasible(&mut self) -> bool {
        return !self.config.use_vcut || !self.vcut();
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        let mut row = String::new();
        for u in 0..self.len() {
            if u % self.w == 0 && u > 0 {
                result.push_str(&format!("\n{}\n", row));
                row = String::new();
            }

            let color = match self.dsu.get_data(u) {
                Link::Complete(color) => color,
                Link::Colored(color, _) => color,
                Link::Uncolored(_, _) => 0,
            };

            result.push_str(&format!("{:02}", color));
            result.push(if self.right[u] { '-' } else { ' ' });

            row.push_str(if self.down[u] { " | " } else { "   " });
        }
        return result;
    }

    pub fn edges(&self) -> (usize, usize, Vec<bool>, Vec<bool>) {
        (self.h, self.w, self.down.clone(), self.right.clone())
    }

    pub fn colors(&self) -> Vec<usize> {
        (0..self.len()).map(|u| {
            match self.dsu.get_data(u) {
                Link::Complete(color) => color,
                Link::Colored(color, _) => color,
                Link::Uncolored(_, _) => 0,
            }
        }).collect()
    }
}
