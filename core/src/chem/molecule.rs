use super::element;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    Single,
    Double,
    Triple,
    Quadruple,
    Aromatic,
}

impl Order {
    pub fn count(self) -> u8 {
        match self {
            Order::Single | Order::Aromatic => 1,
            Order::Double => 2,
            Order::Triple => 3,
            Order::Quadruple => 4,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub number: u8,
    pub charge: i8,
    pub isotope: u16,
    pub aromatic: bool,
    pub bracket: bool,
    pub hydrogens: u8,
    pub map: u16,
}

impl Atom {
    pub fn new(number: u8) -> Atom {
        Atom {
            number,
            charge: 0,
            isotope: 0,
            aromatic: false,
            bracket: false,
            hydrogens: 0,
            map: 0,
        }
    }

    pub fn symbol(&self) -> &'static str {
        element::by_number(self.number)
            .map(|e| e.symbol)
            .unwrap_or("?")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Bond {
    pub from: usize,
    pub to: usize,
    pub order: Order,
    pub ring: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Molecule {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
    neighbours: Vec<Vec<usize>>,
}

fn standard_valences(number: u8) -> &'static [i16] {
    match number {
        1 => &[1],
        5 => &[3],
        6 => &[4],
        7 | 15 => &[3, 5],
        8 => &[2],
        16 | 34 => &[2, 4, 6],
        9 | 17 | 35 | 53 => &[1],
        _ => &[],
    }
}

fn charge_shift(number: u8, charge: i8) -> i16 {
    match number {
        5 => -(charge as i16),
        6 => -(charge as i16).abs(),
        7 | 8 | 15 | 16 | 34 | 9 | 17 | 35 | 53 => charge as i16,
        _ => 0,
    }
}

impl Molecule {
    pub fn add_atom(&mut self, atom: Atom) -> usize {
        self.atoms.push(atom);
        self.neighbours.push(Vec::new());
        self.atoms.len() - 1
    }

    pub fn add_bond(&mut self, from: usize, to: usize, order: Order) {
        if from == to || from >= self.atoms.len() || to >= self.atoms.len() {
            return;
        }
        self.bonds.push(Bond {
            from,
            to,
            order,
            ring: false,
        });
        self.neighbours[from].push(to);
        self.neighbours[to].push(from);
    }

    pub fn neighbours(&self, atom: usize) -> &[usize] {
        &self.neighbours[atom]
    }

    pub fn degree(&self, atom: usize) -> usize {
        self.neighbours[atom].len()
    }

    pub fn bonds_at(&self, atom: usize) -> impl Iterator<Item = &Bond> {
        self.bonds
            .iter()
            .filter(move |bond| bond.from == atom || bond.to == atom)
    }

    pub fn bond_between(&self, a: usize, b: usize) -> Option<&Bond> {
        self.bonds
            .iter()
            .find(|bond| (bond.from == a && bond.to == b) || (bond.from == b && bond.to == a))
    }

    pub fn bond_order_sum(&self, atom: usize) -> i16 {
        self.bonds_at(atom)
            .map(|bond| bond.order.count() as i16)
            .sum()
    }

    pub fn is_aromatic(&self, atom: usize) -> bool {
        self.atoms[atom].aromatic
            || self
                .bonds_at(atom)
                .any(|bond| bond.order == Order::Aromatic)
    }

    pub fn fill_hydrogens(&mut self) {
        for index in 0..self.atoms.len() {
            if self.atoms[index].bracket {
                continue;
            }
            let atom = self.atoms[index];
            let shift = charge_shift(atom.number, atom.charge);
            let options = standard_valences(atom.number);
            let plain = self.bond_order_sum(index) + atom.hydrogens as i16;
            let lowest = match options.first() {
                Some(first) => first + shift,
                None => {
                    self.atoms[index].hydrogens = atom.hydrogens;
                    continue;
                }
            };
            let used = if self.is_aromatic(index) && plain < lowest {
                plain + 1
            } else {
                plain
            };
            let target = options
                .iter()
                .map(|value| value + shift)
                .find(|value| *value >= used)
                .unwrap_or(used);
            self.atoms[index].hydrogens = (target - used).max(0) as u8;
        }
    }

    pub fn heavy_atoms(&self) -> usize {
        self.atoms.iter().filter(|a| a.number != 1).count()
    }

    pub fn total_hydrogens(&self) -> usize {
        self.atoms
            .iter()
            .map(|a| a.hydrogens as usize + usize::from(a.number == 1))
            .sum()
    }

    pub fn components(&self) -> usize {
        let mut seen = vec![false; self.atoms.len()];
        let mut count = 0;
        for start in 0..self.atoms.len() {
            if seen[start] {
                continue;
            }
            count += 1;
            let mut stack = vec![start];
            seen[start] = true;
            while let Some(current) = stack.pop() {
                for &next in &self.neighbours[current] {
                    if !seen[next] {
                        seen[next] = true;
                        stack.push(next);
                    }
                }
            }
        }
        count
    }

    pub fn ring_count(&self) -> usize {
        if self.atoms.is_empty() {
            return 0;
        }
        self.bonds.len() + self.components() - self.atoms.len()
    }

    pub fn mark_rings(&mut self) {
        let bridges = self.bridges();
        for (index, bond) in self.bonds.iter_mut().enumerate() {
            bond.ring = !bridges[index];
        }
    }

    fn bridges(&self) -> Vec<bool> {
        let count = self.atoms.len();
        let mut is_bridge = vec![false; self.bonds.len()];
        let mut discovered = vec![usize::MAX; count];
        let mut low = vec![usize::MAX; count];
        let mut clock = 0usize;

        let mut incident: Vec<Vec<(usize, usize)>> = vec![Vec::new(); count];
        for (index, bond) in self.bonds.iter().enumerate() {
            incident[bond.from].push((bond.to, index));
            incident[bond.to].push((bond.from, index));
        }

        for root in 0..count {
            if discovered[root] != usize::MAX {
                continue;
            }
            let mut stack: Vec<(usize, usize, usize)> = vec![(root, usize::MAX, 0)];
            discovered[root] = clock;
            low[root] = clock;
            clock += 1;

            while let Some(&mut (node, parent_edge, ref mut cursor)) = stack.last_mut() {
                if *cursor < incident[node].len() {
                    let (next, edge) = incident[node][*cursor];
                    *cursor += 1;
                    if edge == parent_edge {
                        continue;
                    }
                    if discovered[next] == usize::MAX {
                        discovered[next] = clock;
                        low[next] = clock;
                        clock += 1;
                        stack.push((next, edge, 0));
                    } else {
                        low[node] = low[node].min(discovered[next]);
                    }
                } else {
                    stack.pop();
                    if let Some(&(above, _, _)) = stack.last() {
                        low[above] = low[above].min(low[node]);
                        if low[node] > discovered[above] {
                            is_bridge[parent_edge] = true;
                        }
                    }
                }
            }
        }

        is_bridge
    }

    pub fn in_ring(&self, atom: usize) -> bool {
        self.bonds_at(atom).any(|bond| bond.ring)
    }

    pub fn smallest_ring(&self, atom: usize) -> Option<usize> {
        if !self.in_ring(atom) {
            return None;
        }
        let count = self.atoms.len();
        let mut best: Option<usize> = None;

        for &start in &self.neighbours[atom] {
            let mut distance = vec![usize::MAX; count];
            let mut queue = std::collections::VecDeque::new();
            distance[atom] = 0;
            distance[start] = 1;
            queue.push_back(start);

            while let Some(current) = queue.pop_front() {
                for &next in &self.neighbours[current] {
                    if next == atom {
                        if current != start {
                            let length = distance[current] + 1;
                            best = Some(best.map_or(length, |b: usize| b.min(length)));
                        }
                        continue;
                    }
                    if distance[next] == usize::MAX {
                        distance[next] = distance[current] + 1;
                        queue.push_back(next);
                    }
                }
            }
        }

        best
    }

    pub fn aromatic_ring_count(&self) -> usize {
        let mut seen = vec![false; self.atoms.len()];
        let mut rings = 0;
        for index in 0..self.atoms.len() {
            if seen[index] || !self.atoms[index].aromatic || !self.in_ring(index) {
                continue;
            }
            let mut stack = vec![index];
            seen[index] = true;
            let mut members = 0;
            let mut inner = 0;
            while let Some(current) = stack.pop() {
                members += 1;
                for &next in &self.neighbours[current] {
                    if self.atoms[next].aromatic {
                        inner += 1;
                        if !seen[next] {
                            seen[next] = true;
                            stack.push(next);
                        }
                    }
                }
            }
            rings += (inner / 2) + 1 - members;
        }
        rings
    }

    pub fn rotatable_bonds(&self) -> usize {
        self.bonds
            .iter()
            .filter(|bond| {
                bond.order == Order::Single
                    && !bond.ring
                    && self.degree(bond.from) > 1
                    && self.degree(bond.to) > 1
                    && !self.is_amide(bond)
            })
            .count()
    }

    fn is_amide(&self, bond: &Bond) -> bool {
        let pair = [(bond.from, bond.to), (bond.to, bond.from)];
        pair.iter().any(|&(carbon, nitrogen)| {
            self.atoms[carbon].number == 6
                && self.atoms[nitrogen].number == 7
                && self.bonds_at(carbon).any(|other| {
                    other.order == Order::Double && {
                        let far = if other.from == carbon {
                            other.to
                        } else {
                            other.from
                        };
                        self.atoms[far].number == 8
                    }
                })
        })
    }
}
