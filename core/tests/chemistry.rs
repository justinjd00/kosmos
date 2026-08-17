use kosmos_core::chem::molecule::Order;
use kosmos_core::chem::smiles::parse;

fn formula(source: &str) -> String {
    let m = parse(source).unwrap_or_else(|e| panic!("{source}: {e}"));
    let mut counts = std::collections::BTreeMap::new();
    for atom in &m.atoms {
        *counts.entry(atom.symbol()).or_insert(0usize) += 1;
    }
    let h = m.total_hydrogens();
    let mut out = String::new();
    if let Some(c) = counts.remove("C") {
        out.push_str(&format!(
            "C{}",
            if c > 1 { c.to_string() } else { String::new() }
        ));
        if h > 0 {
            out.push_str(&format!(
                "H{}",
                if h > 1 { h.to_string() } else { String::new() }
            ));
        }
    }
    for (symbol, count) in counts {
        out.push_str(symbol);
        if count > 1 {
            out.push_str(&count.to_string());
        }
    }
    out
}

#[test]
fn organic_subset() {
    assert_eq!(formula("C"), "CH4");
    assert_eq!(formula("CC"), "C2H6");
    assert_eq!(formula("C=C"), "C2H4");
    assert_eq!(formula("C#C"), "C2H2");
    assert_eq!(formula("CCO"), "C2H6O");
    assert_eq!(formula("CC(=O)O"), "C2H4O2");
    assert_eq!(formula("ClC(Cl)(Cl)Cl"), "CCl4");
}

#[test]
fn aromatic_rings() {
    assert_eq!(formula("c1ccccc1"), "C6H6");
    assert_eq!(formula("C1=CC=CC=C1"), "C6H6");
    assert_eq!(formula("Cc1ccccc1"), "C7H8");
    assert_eq!(formula("c1ccncc1"), "C5H5N");
    assert_eq!(formula("c1cc[nH]c1"), "C4H5N");
}

#[test]
fn real_molecules() {
    assert_eq!(formula("CC(=O)Oc1ccccc1C(=O)O"), "C9H8O4");
    assert_eq!(formula("CN1C=NC2=C1C(=O)N(C)C(=O)N2C"), "C8H10N4O2");
    assert_eq!(formula("CC(C)Cc1ccc(cc1)C(C)C(=O)O"), "C13H18O2");
    assert_eq!(formula("OCC1OC(O)C(O)C(O)C1O"), "C6H12O6");
    assert_eq!(formula("CC(N)C(=O)O"), "C3H7NO2");
}

#[test]
fn brackets_and_charges() {
    let m = parse("[Na+].[Cl-]").unwrap();
    assert_eq!(m.atoms.len(), 2);
    assert_eq!(m.atoms[0].charge, 1);
    assert_eq!(m.atoms[1].charge, -1);
    assert_eq!(m.components(), 2);

    let m = parse("[13CH4]").unwrap();
    assert_eq!(m.atoms[0].isotope, 13);
    assert_eq!(m.atoms[0].hydrogens, 4);

    let m = parse("[NH4+]").unwrap();
    assert_eq!(m.atoms[0].hydrogens, 4);
    assert_eq!(m.atoms[0].charge, 1);
}

#[test]
fn rings_are_found() {
    let benzene = parse("c1ccccc1").unwrap();
    assert_eq!(benzene.ring_count(), 1);
    assert!(benzene.bonds.iter().all(|b| b.ring));
    assert_eq!(benzene.smallest_ring(0), Some(6));

    let naphthalene = parse("c1ccc2ccccc2c1").unwrap();
    assert_eq!(naphthalene.ring_count(), 2);
    assert_eq!(naphthalene.aromatic_ring_count(), 2);

    let toluene = parse("Cc1ccccc1").unwrap();
    assert_eq!(toluene.ring_count(), 1);
    assert!(!toluene.in_ring(0));

    let caffeine = parse("CN1C=NC2=C1C(=O)N(C)C(=O)N2C").unwrap();
    assert_eq!(caffeine.ring_count(), 2);

    let cyclopropane = parse("C1CC1").unwrap();
    assert_eq!(cyclopropane.smallest_ring(0), Some(3));
}

#[test]
fn bond_orders_survive() {
    let m = parse("CC(=O)O").unwrap();
    assert_eq!(
        m.bonds.iter().filter(|b| b.order == Order::Double).count(),
        1
    );
    let m = parse("c1ccccc1").unwrap();
    assert_eq!(
        m.bonds
            .iter()
            .filter(|b| b.order == Order::Aromatic)
            .count(),
        6
    );
}

#[test]
fn rotatable_bonds_are_counted() {
    assert_eq!(parse("CCCC").unwrap().rotatable_bonds(), 1);
    assert_eq!(parse("c1ccccc1").unwrap().rotatable_bonds(), 0);
    assert_eq!(parse("CC(=O)Oc1ccccc1C(=O)O").unwrap().rotatable_bonds(), 3);
}

#[test]
fn bad_input_is_reported() {
    for source in ["", "C1CC", "CC)", "(CC", "[Xx]", "C%1", "$$$"] {
        assert!(parse(source).is_err(), "{source:?} should not parse");
    }
    let error = parse("CC$$").unwrap_err();
    assert!(error.at > 0);
}
