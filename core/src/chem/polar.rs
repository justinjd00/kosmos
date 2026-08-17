use super::molecule::{Bond, Molecule, Order};
use std::collections::HashSet;
use std::collections::VecDeque;

const HYDROGEN: u8 = 1;
const CARBON: u8 = 6;
const NITROGEN: u8 = 7;
const OXYGEN: u8 = 8;
const PHOSPHORUS: u8 = 15;
const SULFUR: u8 = 16;

/// Topological polar surface area in square angstroms, after
/// Ertl, Rohde and Selzer, J. Med. Chem. 2000, 43, 3714-3717.
///
/// Every nitrogen and oxygen is classified by its element, attached
/// hydrogens, formal charge, membership of a three-membered ring and the
/// orders of its bonds to heavy neighbours, then looked up in Table 1 of
/// the paper. A nitrogen or oxygen in an environment the table does not
/// list falls back to the paper's fitted equations,
/// 30.5 - 8.2 n + 1.5 nH for nitrogen and 28.5 - 8.6 n + 1.5 nH for
/// oxygen, where n counts heavy neighbours; the result is clamped at
/// zero. Sulfur and phosphorus are ignored here, as in the paper's
/// principal definition.
pub fn surface(molecule: &Molecule) -> f64 {
    total(molecule, false)
}

/// The same, with the sulfur and phosphorus contributions included.
///
/// Adds the eleven sulfur and phosphorus rows of Table 1 of Ertl, Rohde
/// and Selzer, J. Med. Chem. 2000, 43, 3714-3717. Sulfur and phosphorus
/// in an environment the table does not list contribute nothing, since
/// the paper fits its fallback equations for nitrogen and oxygen only.
pub fn surface_with_sulfur_and_phosphorus(molecule: &Molecule) -> f64 {
    total(molecule, true)
}

fn total(molecule: &Molecule, sulfur_and_phosphorus: bool) -> f64 {
    let aromatic = aromatic_atoms(molecule);
    (0..molecule.atoms.len())
        .map(|index| contribution(molecule, &aromatic, index, sulfur_and_phosphorus))
        .sum()
}

fn contribution(
    molecule: &Molecule,
    aromatic: &[bool],
    index: usize,
    sulfur_and_phosphorus: bool,
) -> f64 {
    let atom = molecule.atoms[index];
    let wanted = match atom.number {
        NITROGEN | OXYGEN => true,
        PHOSPHORUS | SULFUR => sulfur_and_phosphorus,
        _ => false,
    };
    if !wanted {
        return 0.0;
    }

    let (single, double, triple, resonant) = environment(molecule, aromatic, index);
    let hydrogens = atom.hydrogens + attached_hydrogens(molecule, index);
    let strained = molecule.smallest_ring(index) == Some(3);

    match (
        atom.number,
        atom.charge,
        hydrogens,
        (single, double, triple, resonant),
    ) {
        (NITROGEN, 0, 0, (3, 0, 0, 0)) if strained => 3.01,
        (NITROGEN, 0, 1, (2, 0, 0, 0)) if strained => 21.94,
        (OXYGEN, 0, 0, (2, 0, 0, 0)) if strained => 12.53,

        (NITROGEN, 0, 0, (3, 0, 0, 0)) => 3.24,
        (NITROGEN, 0, 0, (1, 1, 0, 0)) => 12.36,
        (NITROGEN, 0, 0, (0, 0, 1, 0)) => 23.79,
        (NITROGEN, 0, 0, (1, 2, 0, 0)) => 11.68,
        (NITROGEN, 0, 0, (0, 1, 1, 0)) => 13.60,
        (NITROGEN, 0, 1, (2, 0, 0, 0)) => 12.03,
        (NITROGEN, 0, 1, (0, 1, 0, 0)) => 23.85,
        (NITROGEN, 0, 2, (1, 0, 0, 0)) => 26.02,

        (NITROGEN, 1, 0, (4, 0, 0, 0)) => 0.00,
        (NITROGEN, 1, 0, (2, 1, 0, 0)) => 3.01,
        (NITROGEN, 1, 0, (1, 0, 1, 0)) => 4.36,
        (NITROGEN, 1, 1, (3, 0, 0, 0)) => 4.44,
        (NITROGEN, 1, 1, (1, 1, 0, 0)) => 13.97,
        (NITROGEN, 1, 2, (2, 0, 0, 0)) => 16.61,
        (NITROGEN, 1, 2, (0, 1, 0, 0)) => 25.59,
        (NITROGEN, 1, 3, (1, 0, 0, 0)) => 27.64,

        (NITROGEN, 0, 0, (0, 0, 0, 2)) => 12.89,
        (NITROGEN, 0, 0, (0, 0, 0, 3)) => 4.41,
        (NITROGEN, 0, 0, (1, 0, 0, 2)) => 4.93,
        (NITROGEN, 0, 0, (0, 1, 0, 2)) => 8.39,
        (NITROGEN, 0, 1, (0, 0, 0, 2)) => 15.79,
        (NITROGEN, 1, 0, (0, 0, 0, 3)) => 4.10,
        (NITROGEN, 1, 0, (1, 0, 0, 2)) => 3.88,
        (NITROGEN, 1, 1, (0, 0, 0, 2)) => 14.14,

        (OXYGEN, 0, 0, (2, 0, 0, 0)) => 9.23,
        (OXYGEN, 0, 0, (0, 1, 0, 0)) => 17.07,
        (OXYGEN, 0, 1, (1, 0, 0, 0)) => 20.23,
        (OXYGEN, -1, 0, (1, 0, 0, 0)) => 23.06,
        (OXYGEN, 0, 0, (0, 0, 0, 2)) => 13.14,

        (SULFUR, 0, 0, (2, 0, 0, 0)) => 25.30,
        (SULFUR, 0, 0, (0, 1, 0, 0)) => 32.09,
        (SULFUR, 0, 0, (2, 1, 0, 0)) => 19.21,
        (SULFUR, 0, 0, (2, 2, 0, 0)) => 8.38,
        (SULFUR, 0, 1, (1, 0, 0, 0)) => 38.80,
        (SULFUR, 0, 0, (0, 0, 0, 2)) => 28.24,
        (SULFUR, 0, 0, (0, 1, 0, 2)) => 21.70,

        (PHOSPHORUS, 0, 0, (3, 0, 0, 0)) => 13.59,
        (PHOSPHORUS, 0, 0, (1, 1, 0, 0)) => 34.14,
        (PHOSPHORUS, 0, 0, (3, 1, 0, 0)) => 9.81,
        (PHOSPHORUS, 0, 1, (2, 1, 0, 0)) => 23.47,

        _ => fitted(atom.number, single + double + triple + resonant, hydrogens),
    }
}

fn fitted(number: u8, heavy: u8, hydrogens: u8) -> f64 {
    let heavy = f64::from(heavy);
    let hydrogens = f64::from(hydrogens);
    match number {
        NITROGEN => (30.5 - 8.2 * heavy + 1.5 * hydrogens).max(0.0),
        OXYGEN => (28.5 - 8.6 * heavy + 1.5 * hydrogens).max(0.0),
        _ => 0.0,
    }
}

fn attached_hydrogens(molecule: &Molecule, index: usize) -> u8 {
    molecule
        .neighbours(index)
        .iter()
        .filter(|&&other| molecule.atoms[other].number == HYDROGEN)
        .count() as u8
}

fn environment(molecule: &Molecule, aromatic: &[bool], index: usize) -> (u8, u8, u8, u8) {
    let mut single = 0;
    let mut double = 0;
    let mut triple = 0;
    let mut resonant = 0;

    for bond in molecule.bonds_at(index) {
        let other = far_end(bond, index);
        if molecule.atoms[other].number == HYDROGEN {
            continue;
        }
        if bond.ring && aromatic[index] && aromatic[other] {
            resonant += 1;
            continue;
        }
        match bond.order {
            Order::Single => single += 1,
            Order::Double => double += 1,
            Order::Triple | Order::Quadruple => triple += 1,
            Order::Aromatic => resonant += 1,
        }
    }

    (single, double, triple, resonant)
}

fn far_end(bond: &Bond, index: usize) -> usize {
    if bond.from == index {
        bond.to
    } else {
        bond.from
    }
}

fn aromatic_atoms(molecule: &Molecule) -> Vec<bool> {
    let mut flags: Vec<bool> = molecule.atoms.iter().map(|atom| atom.aromatic).collect();
    let mut found = vec![false; molecule.atoms.len()];

    for ring in rings(molecule) {
        if ring.iter().any(|&index| flags[index]) {
            continue;
        }
        match pi_electrons(molecule, &ring) {
            Some(count) if count >= 6 && count % 4 == 2 => {
                for index in ring {
                    found[index] = true;
                }
            }
            _ => {}
        }
    }

    for (flag, extra) in flags.iter_mut().zip(found) {
        *flag |= extra;
    }
    flags
}

fn pi_electrons(molecule: &Molecule, ring: &[usize]) -> Option<usize> {
    let mut electrons = 0;

    for &index in ring {
        let atom = molecule.atoms[index];
        let mut inside = false;
        let mut outside = false;
        for bond in molecule.bonds_at(index) {
            match bond.order {
                Order::Double => {
                    if ring.contains(&far_end(bond, index)) {
                        inside = true;
                    } else {
                        outside = true;
                    }
                }
                Order::Triple | Order::Quadruple => return None,
                _ => {}
            }
        }
        electrons += match (atom.number, inside, outside) {
            (CARBON | NITROGEN, true, false) => 1,
            (NITROGEN, false, false) if atom.hydrogens as usize + molecule.degree(index) == 3 => 2,
            (OXYGEN | SULFUR, false, false) if molecule.degree(index) == 2 => 2,
            _ => return None,
        };
    }

    Some(electrons)
}

fn rings(molecule: &Molecule) -> Vec<Vec<usize>> {
    let mut seen: HashSet<Vec<usize>> = HashSet::new();
    let mut found = Vec::new();

    for bond in molecule.bonds.iter().filter(|bond| bond.ring) {
        if let Some(cycle) = smallest_cycle(molecule, bond) {
            let mut key = cycle.clone();
            key.sort_unstable();
            if seen.insert(key) {
                found.push(cycle);
            }
        }
    }

    found
}

fn smallest_cycle(molecule: &Molecule, bond: &Bond) -> Option<Vec<usize>> {
    let count = molecule.atoms.len();
    let mut previous = vec![usize::MAX; count];
    let mut seen = vec![false; count];
    let mut queue = VecDeque::new();

    seen[bond.from] = true;
    queue.push_back(bond.from);
    while let Some(current) = queue.pop_front() {
        for &next in molecule.neighbours(current) {
            if seen[next] || (current == bond.from && next == bond.to) {
                continue;
            }
            seen[next] = true;
            previous[next] = current;
            queue.push_back(next);
        }
    }

    if !seen[bond.to] {
        return None;
    }

    let mut cycle = vec![bond.to];
    let mut current = bond.to;
    while current != bond.from {
        current = previous[current];
        cycle.push(current);
    }
    Some(cycle)
}

#[cfg(test)]
mod tests {
    use super::super::smiles::parse;
    use super::{surface, surface_with_sulfur_and_phosphorus};

    fn area(source: &str) -> f64 {
        surface(&parse(source).unwrap_or_else(|error| panic!("{source}: {error}")))
    }

    fn area_with_sulfur_and_phosphorus(source: &str) -> f64 {
        surface_with_sulfur_and_phosphorus(&parse(source).unwrap())
    }

    fn check(source: &str, expected: f64) {
        let found = area(source);
        assert!(
            (found - expected).abs() < 0.1,
            "{source}: found {found:.2}, expected {expected:.2}"
        );
    }

    #[test]
    fn published_drugs() {
        check("c1ccccc1", 0.00);
        check("CO", 20.23);
        check("CC(=O)O", 37.30);
        check("CC(=O)Oc1ccccc1C(=O)O", 63.60);
        check("CN1C=NC2=C1C(=O)N(C)C(=O)N2C", 58.44);
        check("CC(=O)Nc1ccc(O)cc1", 49.33);
        check("OCC1OC(O)C(O)C(O)C1O", 110.38);
        check("CN1CCC[C@H]1c1cccnc1", 16.13);
        check("CC(N)C(=O)O", 63.32);
        check("c1ccncc1", 12.89);
        check("CC(C)Cc1ccc(cc1)C(C)C(=O)O", 37.30);
    }

    #[test]
    fn small_molecules() {
        check("CN", 26.02);
        check("CNC", 12.03);
        check("CCOCC", 9.23);
        check("CC#N", 23.79);
        check("C[N+](C)(C)C", 0.00);
        check("C[NH3+]", 27.64);
        check("CC(=O)[O-]", 40.13);
        check("CN(=O)=O", 45.82);
        check("C[N+](=O)[O-]", 43.14);
    }

    #[test]
    fn environments_outside_the_table() {
        check("N", 35.00);
        check("O", 31.50);
    }

    #[test]
    fn three_membered_rings() {
        check("C1CO1", 12.53);
        check("C1CN1", 21.94);
        check("C1CC1", 0.00);
    }

    #[test]
    fn aromatic_heterocycles() {
        check("c1cc[nH]c1", 15.79);
        check("c1ccoc1", 13.14);
        check("c1ncc2[nH]cnc2n1", 54.46);
        check("C1=CN=CN1", 28.68);
    }

    fn check_with_sulfur_and_phosphorus(source: &str, expected: f64) {
        let found = area_with_sulfur_and_phosphorus(source);
        assert!(
            (found - expected).abs() < 0.1,
            "{source}: found {found:.2}, expected {expected:.2}"
        );
    }

    #[test]
    fn sulfur_and_phosphorus() {
        check_with_sulfur_and_phosphorus("CSC", 25.30);
        check_with_sulfur_and_phosphorus("CS(C)=O", 36.28);
        check_with_sulfur_and_phosphorus("CS(C)(=O)=O", 42.52);
        check_with_sulfur_and_phosphorus("CS", 38.80);
        check_with_sulfur_and_phosphorus("c1cc[s]c1", 28.24);
        check_with_sulfur_and_phosphorus("CP(C)C", 13.59);
        check_with_sulfur_and_phosphorus("COP(=O)(OC)OC", 54.57);
        check("CS(C)=O", 17.07);
        check("CSC", 0.00);
    }
}
