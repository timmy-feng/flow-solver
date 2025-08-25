#[derive(Debug, Clone, Copy)]
struct Log<T>(usize, usize, usize, T);

pub trait Unite {
    fn unite(self, oth: Self, u: usize, v: usize) -> Option<Self> where Self: Sized;
}

#[derive(Debug, Clone, Copy)]
enum Node {
    Root(usize),
    NonRoot(usize),
}

impl Node {
    fn size(&self) -> usize {
        match self {
            Node::Root(size) => *size,
            _ => panic!("Node was not root"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnionFind<T> {
    nodes: Vec<Node>,
    data: Vec<T>,
    history: Vec<Log<T>>,
}

impl<T> UnionFind<T>
where
    T: Copy + Unite
{
    pub fn from(data: Vec<T>) -> Self {
        Self { nodes: vec![Node::Root(1); data.len()], data, history: Vec::new() }
    }
    
    fn find(&self, mut u: usize) -> usize {
        loop {
            match self.nodes[u] {
                Node::NonRoot(p) => u = p,
                _ => return u,
            }
        }
    }

    pub fn same(&self, u: usize, v: usize) -> bool { self.find(u) == self.find(v) }

    pub fn get_data(&self, u: usize) -> T { self.data[self.find(u)] }

    pub fn unite(&mut self, u: usize, v: usize) -> bool {
        let mut a = self.find(u);
        let mut b = self.find(v);

        if a == b {
            return false;
        }

        if let Some(link) = self.data[a].unite(self.data[b], u, v) {
            if self.nodes[a].size() < self.nodes[b].size() {
                (a, b) = (b, a);
            }

            self.history.push(Log(a, b, self.nodes[b].size(), self.data[a]));
            
            self.nodes[a] = Node::Root(self.nodes[a].size() + self.nodes[b].size());
            self.nodes[b] = Node::NonRoot(a);

            self.data[a] = link;

            return true;
        }
        return false;
    }

    pub fn undo(&mut self) {
        assert!(!self.history.is_empty());

        let Log(u, v, size_v, link_u) = self.history.pop().unwrap();

        self.nodes[u] = Node::Root(self.nodes[u].size() - size_v);
        self.nodes[v] = Node::Root(size_v);

        self.data[u] = link_u;
    }
}