use std::cmp::Reverse;
use std::collections::BinaryHeap;

const NONE: u32 = u32::MAX;
const FAR: i64 = i64::MAX / 4;

pub struct MinCostFlow {
    first: Vec<u32>,
    next: Vec<u32>,
    to: Vec<u32>,
    cap: Vec<i32>,
    cost: Vec<i32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Solved {
    pub flow: i64,
    pub cost: i64,
}

impl MinCostFlow {
    pub fn new(nodes: usize) -> MinCostFlow {
        MinCostFlow { first: vec![NONE; nodes], next: Vec::new(), to: Vec::new(), cap: Vec::new(), cost: Vec::new() }
    }

    pub fn add_arc(&mut self, u: usize, v: usize, cap: i32, cost: i32) -> usize {
        let id = self.to.len();
        self.to.push(v as u32);
        self.cap.push(cap.max(0));
        self.cost.push(cost);
        self.next.push(self.first[u]);
        self.first[u] = id as u32;
        self.to.push(u as u32);
        self.cap.push(0);
        self.cost.push(-cost);
        self.next.push(self.first[v]);
        self.first[v] = id as u32 + 1;
        id
    }

    pub fn flow_on(&self, arc: usize) -> i32 {
        self.cap[arc ^ 1]
    }

    pub fn solve(&mut self, source: usize, sink: usize, want: i64) -> Solved {
        let n = self.first.len();
        let mut pot = vec![0i64; n];
        let mut dist = vec![FAR; n];
        let mut back = vec![NONE; n];
        let mut done = Solved::default();
        while done.flow < want {
            dist.fill(FAR);
            back.fill(NONE);
            dist[source] = 0;
            let mut heap: BinaryHeap<Reverse<(i64, u32)>> = BinaryHeap::new();
            heap.push(Reverse((0, source as u32)));
            let mut reach = FAR;
            while let Some(Reverse((d, u))) = heap.pop() {
                if d > dist[u as usize] {
                    continue;
                }
                if u as usize == sink {
                    reach = d;
                    break;
                }
                let mut a = self.first[u as usize];
                while a != NONE {
                    let ai = a as usize;
                    if self.cap[ai] > 0 {
                        let v = self.to[ai] as usize;
                        let nd = d + self.cost[ai] as i64 + pot[u as usize] - pot[v];
                        if nd < dist[v] {
                            dist[v] = nd;
                            back[v] = a;
                            heap.push(Reverse((nd, v as u32)));
                        }
                    }
                    a = self.next[ai];
                }
            }
            if reach >= FAR {
                break;
            }
            for v in 0..n {
                pot[v] += dist[v].min(reach);
            }
            let mut push = want - done.flow;
            let mut v = sink;
            while v != source {
                let a = back[v] as usize;
                push = push.min(self.cap[a] as i64);
                v = self.to[a ^ 1] as usize;
            }
            let mut v = sink;
            let mut unit = 0i64;
            while v != source {
                let a = back[v] as usize;
                self.cap[a] -= push as i32;
                self.cap[a ^ 1] += push as i32;
                unit += self.cost[a] as i64;
                v = self.to[a ^ 1] as usize;
            }
            done.flow += push;
            done.cost += push * unit;
        }
        done
    }
}
