use super::*;

#[test]
fn fold() {
    assert_eq!(Core::fold(44, 8000, 8000), 44);
    assert_eq!(Core::fold(8060, 8000, 8000), 60);
}

#[test]
fn build_and_run_imp() {
    let imp = Warrior::parse(include_str!("../../warriors/imp.red")).unwrap();
    let warriors = vec![imp];

    let mut cb = CoreBuilder::new();
    let mut core = cb
        .core_size(10)
        .read_distance(10)
        .write_distance(10)
        .separation(Separation::Fixed(10))
        .load_warriors(&warriors)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I $0, $1",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );

    core.run_once();
    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );

    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();
    core.run_once();

    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1",
            "MOV.I $0, $1"
        ]
    );
}

#[test]
fn imp_fight() {
    let imp = Warrior::parse(include_str!("../../warriors/imp.red")).unwrap();
    let imp2 = imp.clone();
    let warriors = vec![imp.clone(), imp2.clone()];

    let mut cb = CoreBuilder::new();
    let mut core = cb
        .separation(Separation::Fixed(500))
        .load_warriors(&warriors)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(core.run(), MatchOutcome::Draw(vec![&imp, &imp2]));
}

#[test]
fn build_and_run_dwarf() {
    let dwarf = Warrior::parse(include_str!("../../warriors/dwarf.red")).unwrap();
    let warriors = vec![dwarf.clone()];

    let mut cb = CoreBuilder::new();
    let mut core = cb
        .core_size(10)
        .read_distance(10)
        .write_distance(10)
        .separation(Separation::Fixed(10))
        .load_warriors(&warriors)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "DAT.F #0, #0",
            "ADD.AB #4, $9",
            "MOV.AB #0, @8",
            "JMP.A $8, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );

    core.run_once();
    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "DAT.F #0, #4",
            "ADD.AB #4, $9",
            "MOV.AB #0, @8",
            "JMP.A $8, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );
}

#[test]
fn build_and_run_stone() {
    let stone = Warrior::parse(include_str!("../../warriors/stone.red")).unwrap();
    let warriors = vec![stone.clone()];

    let mut cb = CoreBuilder::new();
    let mut core = cb
        .core_size(10)
        .read_distance(10)
        .write_distance(10)
        .separation(Separation::Fixed(10))
        .load_warriors(&warriors)
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I <2, $3",
            "ADD.F $3, $9",
            "JMP.B $8, $0",
            "DAT.F #0, $0",
            "DAT.F #6, #4",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );

    core.run_once();
    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I <2, $3",
            "ADD.F $3, $9",
            "JMP.B $8, $9",
            "ADD.F $3, $9",
            "DAT.F #6, #4",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );

    core.run_once();
    assert_eq!(
        core.instructions
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>(),
        vec![
            "MOV.I <8, $7",
            "ADD.F $3, $9",
            "JMP.B $8, $9",
            "ADD.F $3, $9",
            "DAT.F #6, #4",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0",
            "DAT.F $0, $0"
        ]
    );
}

#[test]
fn wait_vs_armadillo() {
    let armadillo = Warrior::parse(include_str!("../../warriors/armadillo.red")).unwrap();
    let wait = Warrior::parse(include_str!("../../warriors/wait.red")).unwrap();
    let warriors = vec![armadillo.clone(), wait.clone()];

    let logger = crate::logger::DebugLogger::new();

    let mut cb = CoreBuilder::new();
    let mut core = cb
        .separation(Separation::Fixed(4000))
        .load_warriors(&warriors)
        .unwrap()
        .log_with(Box::new(logger))
        .build()
        .unwrap();

    assert_eq!(core.run(), MatchOutcome::Win(&armadillo));
}

#[test]
fn stone_vs_imp() {
    let stone = Warrior::parse(include_str!("../../warriors/stone.red")).unwrap();
    let imp = Warrior::parse(include_str!("../../warriors/imp.red")).unwrap();
    let warriors = vec![stone.clone(), imp.clone()];

    let logger = crate::logger::DebugLogger::new();

    let mut cb = CoreBuilder::new();
    let core = cb
        .separation(Separation::Fixed(4000))
        .load_warriors(&warriors)
        .unwrap()
        .log_with(Box::new(logger));

    for _ in 0..=10 {
        let mut core = core.build().unwrap();
        core.run();
    }
}
