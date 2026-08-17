use super::element;
use super::molecule::{Atom, Molecule, Order};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub at: usize,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {})", self.message, self.at)
    }
}

type Result<T> = std::result::Result<T, Error>;

const AROMATIC: &[(&str, u8)] = &[
    ("c", 6),
    ("n", 7),
    ("o", 8),
    ("s", 16),
    ("p", 15),
    ("b", 5),
    ("se", 34),
    ("as", 33),
];

const ORGANIC: &[(&str, u8)] = &[
    ("Cl", 17),
    ("Br", 35),
    ("B", 5),
    ("C", 6),
    ("N", 7),
    ("O", 8),
    ("P", 15),
    ("S", 16),
    ("F", 9),
    ("I", 53),
    ("*", 0),
];

struct Reader<'a> {
    text: &'a [u8],
    at: usize,
    molecule: Molecule,
    stack: Vec<usize>,
    previous: Option<usize>,
    pending: Option<Order>,
    rings: Vec<Option<(usize, Option<Order>, usize)>>,
}

fn fail<T>(message: &str, at: usize) -> Result<T> {
    Err(Error {
        message: message.to_string(),
        at,
    })
}

impl<'a> Reader<'a> {
    fn new(text: &'a str) -> Reader<'a> {
        Reader {
            text: text.as_bytes(),
            at: 0,
            molecule: Molecule::default(),
            stack: Vec::new(),
            previous: None,
            pending: None,
            rings: vec![None; 100],
        }
    }

    fn peek(&self) -> Option<u8> {
        self.text.get(self.at).copied()
    }

    fn take(&mut self) -> Option<u8> {
        let value = self.peek();
        if value.is_some() {
            self.at += 1;
        }
        value
    }

    fn digits(&mut self, most: usize) -> u32 {
        let mut value = 0u32;
        let mut taken = 0;
        while taken < most {
            match self.peek() {
                Some(byte) if byte.is_ascii_digit() => {
                    value = value * 10 + u32::from(byte - b'0');
                    self.at += 1;
                    taken += 1;
                }
                _ => break,
            }
        }
        value
    }

    fn attach(&mut self, index: usize) {
        if let Some(previous) = self.previous {
            let order = self.pending.take().unwrap_or_else(|| {
                if self.molecule.atoms[previous].aromatic && self.molecule.atoms[index].aromatic {
                    Order::Aromatic
                } else {
                    Order::Single
                }
            });
            self.molecule.add_bond(previous, index, order);
        } else {
            self.pending = None;
        }
        self.previous = Some(index);
    }

    fn organic(&mut self) -> Result<bool> {
        for (symbol, number) in ORGANIC {
            if self.text[self.at..].starts_with(symbol.as_bytes()) {
                self.at += symbol.len();
                let mut atom = Atom::new(*number);
                if *number == 0 {
                    atom.bracket = true;
                }
                let index = self.molecule.add_atom(atom);
                self.attach(index);
                return Ok(true);
            }
        }
        for (symbol, number) in AROMATIC {
            if self.text[self.at..].starts_with(symbol.as_bytes()) {
                self.at += symbol.len();
                let mut atom = Atom::new(*number);
                atom.aromatic = true;
                let index = self.molecule.add_atom(atom);
                self.attach(index);
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn bracket(&mut self) -> Result<()> {
        let start = self.at;
        self.at += 1;

        let isotope = self.digits(4) as u16;

        let mut aromatic = false;
        let mut symbol = String::new();
        match self.peek() {
            Some(byte) if byte.is_ascii_uppercase() => {
                symbol.push(byte as char);
                self.at += 1;
                if let Some(next) = self.peek() {
                    if next.is_ascii_lowercase() {
                        let guess = format!("{symbol}{}", next as char);
                        if element::by_symbol(&guess).is_some() {
                            symbol = guess;
                            self.at += 1;
                        }
                    }
                }
            }
            Some(byte) if byte.is_ascii_lowercase() => {
                aromatic = true;
                symbol.push(byte as char);
                self.at += 1;
                if self.peek() == Some(b'e') && (symbol == "s" || symbol == "a") {
                    symbol.push('e');
                    self.at += 1;
                }
                symbol = symbol.to_uppercase();
                if symbol.len() == 2 {
                    symbol = format!("{}{}", &symbol[0..1], symbol[1..2].to_lowercase());
                }
            }
            Some(b'*') => {
                symbol.push('*');
                self.at += 1;
            }
            _ => return fail("expected an element symbol", self.at),
        }

        let number = if symbol == "*" {
            0
        } else {
            match element::by_symbol(&symbol) {
                Some(found) => found.number,
                None => return fail(&format!("unknown element '{symbol}'"), start + 1),
            }
        };

        while self.peek() == Some(b'@') {
            self.at += 1;
        }

        let mut hydrogens = 0u8;
        if self.peek() == Some(b'H') {
            self.at += 1;
            let written = self.digits(1);
            hydrogens = if written == 0 { 1 } else { written as u8 };
        }

        let mut charge = 0i8;
        loop {
            match self.peek() {
                Some(b'+') => {
                    self.at += 1;
                    let amount = self.digits(2);
                    charge += if amount == 0 { 1 } else { amount as i8 };
                }
                Some(b'-') => {
                    self.at += 1;
                    let amount = self.digits(2);
                    charge -= if amount == 0 { 1 } else { amount as i8 };
                }
                _ => break,
            }
        }

        let mut map = 0u16;
        if self.peek() == Some(b':') {
            self.at += 1;
            map = self.digits(4) as u16;
        }

        if self.take() != Some(b']') {
            return fail("missing ]", self.at);
        }

        let index = self.molecule.add_atom(Atom {
            number,
            charge,
            isotope,
            aromatic,
            bracket: true,
            hydrogens,
            map,
        });
        self.attach(index);
        Ok(())
    }

    fn ring(&mut self, label: usize) -> Result<()> {
        let current = match self.previous {
            Some(index) => index,
            None => return fail("a ring bond needs an atom before it", self.at),
        };
        let order = self.pending.take();

        match self.rings[label].take() {
            None => self.rings[label] = Some((current, order, self.at)),
            Some((other, other_order, _)) => {
                if other == current {
                    return fail("a ring bond cannot close on itself", self.at);
                }
                let chosen = order.or(other_order).unwrap_or_else(|| {
                    if self.molecule.atoms[current].aromatic && self.molecule.atoms[other].aromatic
                    {
                        Order::Aromatic
                    } else {
                        Order::Single
                    }
                });
                self.molecule.add_bond(other, current, chosen);
            }
        }
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        while let Some(byte) = self.peek() {
            match byte {
                b'[' => self.bracket()?,
                b'(' => {
                    self.at += 1;
                    match self.previous {
                        Some(index) => self.stack.push(index),
                        None => return fail("a branch needs an atom before it", self.at),
                    }
                }
                b')' => {
                    self.at += 1;
                    match self.stack.pop() {
                        Some(index) => self.previous = Some(index),
                        None => return fail("unmatched )", self.at),
                    }
                }
                b'-' => {
                    self.at += 1;
                    self.pending = Some(Order::Single);
                }
                b'=' => {
                    self.at += 1;
                    self.pending = Some(Order::Double);
                }
                b'#' => {
                    self.at += 1;
                    self.pending = Some(Order::Triple);
                }
                b'$' => {
                    self.at += 1;
                    self.pending = Some(Order::Quadruple);
                }
                b':' => {
                    self.at += 1;
                    self.pending = Some(Order::Aromatic);
                }
                b'/' | b'\\' => {
                    self.at += 1;
                    self.pending = Some(Order::Single);
                }
                b'.' => {
                    self.at += 1;
                    self.previous = None;
                    self.pending = None;
                }
                b'%' => {
                    self.at += 1;
                    let label = self.digits(2) as usize;
                    if label < 10 {
                        return fail("%% needs two digits", self.at);
                    }
                    self.ring(label)?;
                }
                byte if byte.is_ascii_digit() => {
                    self.at += 1;
                    self.ring(usize::from(byte - b'0'))?;
                }
                byte if byte.is_ascii_whitespace() => {
                    self.at += 1;
                }
                _ => {
                    let before = self.at;
                    if !self.organic()? {
                        return fail(&format!("unexpected character '{}'", byte as char), before);
                    }
                }
            }
        }

        if let Some(position) = self.rings.iter().flatten().map(|entry| entry.2).next() {
            return fail("a ring bond was never closed", position);
        }
        if !self.stack.is_empty() {
            return fail("unmatched (", self.at);
        }
        if self.pending.is_some() {
            return fail("a bond symbol with nothing after it", self.at);
        }
        Ok(())
    }
}

pub fn parse(source: &str) -> Result<Molecule> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return fail("nothing to read", 0);
    }
    if trimmed.len() > 4096 {
        return fail("that is longer than this parser accepts", 4096);
    }

    let mut reader = Reader::new(trimmed);
    reader.run()?;

    let mut molecule = std::mem::take(&mut reader.molecule);
    if molecule.atoms.is_empty() {
        return fail("no atoms", 0);
    }
    molecule.fill_hydrogens();
    molecule.mark_rings();
    Ok(molecule)
}
