pub struct BipartiteGraph {
    adj_u: Vec<Vec<usize>>,
    match_u: Vec<Option<usize>>,
    match_v: Vec<Option<usize>>,
    n: usize,
    m: usize,
}

impl BipartiteGraph {
    pub fn new(n: usize, m: usize) -> Self {
        Self {
            adj_u: (0..n).map(|_| Vec::new()).collect(),
            match_u: vec![None; n],
            match_v: vec![None; m],
            n, m,
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        assert!(u < self.n && v < self.m);
        self.adj_u[u].push(v)
    }

    fn kuhn_push(&mut self, u: usize, visited: &mut [bool]) -> bool {
        visited[u] = true;
        for i in 0..self.adj_u[u].len() {
            let v = self.adj_u[u][i];
            match self.match_v[v] {
                None => {
                    self.match_v[v] = Some(u);
                    self.match_u[u] = Some(v);
                    return true;
                },
                Some(w) => {
                    if !visited[w] && self.kuhn_push(w, visited) {
                        self.match_v[v] = Some(u);
                        self.match_u[u] = Some(v);
                        return true;
                    }
                }
            }
        }
        return false;
    }

    pub fn kuhn_matching(&mut self) -> usize {
        let mut result = 0;
        for u in 0..self.n {
            for &v in self.adj_u[u].iter() {
                if self.match_v[v].is_none() {
                    self.match_u[u] = Some(v);
                    self.match_v[v] = Some(u);
                    result += 1;
                    break;
                }
            }
        }

        for u in 0..self.n {
            if self.match_u[u].is_none() && self.kuhn_push(u, &mut vec![false; self.n]) {
                result += 1;
            }
        }
        return result;
    }
}