use cccp::ddtdd::{parse_phase_arg, DdtddPhase};

#[test]
fn test_parse_phase_arg_numeric_valid() {
    assert_eq!(parse_phase_arg("3").unwrap(), DdtddPhase::Ph3);
    assert_eq!(parse_phase_arg("4").unwrap(), DdtddPhase::Ph4);
    assert_eq!(parse_phase_arg("5").unwrap(), DdtddPhase::Ph5);
    assert_eq!(parse_phase_arg("6").unwrap(), DdtddPhase::Ph6);
    assert_eq!(parse_phase_arg("7").unwrap(), DdtddPhase::Ph7);
    assert_eq!(parse_phase_arg("8").unwrap(), DdtddPhase::Ph8);
}

#[test]
fn test_parse_phase_arg_reject_phase_1_and_2() {
    let err1 = parse_phase_arg("1").unwrap_err();
    assert!(
        err1.contains("não possuem commits"),
        "Fase 1 deve ser rejeitada com explicação"
    );
    let err2 = parse_phase_arg("2").unwrap_err();
    assert!(
        err2.contains("não possuem commits"),
        "Fase 2 deve ser rejeitada com explicação"
    );
}

#[test]
fn test_parse_phase_arg_reject_phase_9() {
    let err9 = parse_phase_arg("9").unwrap_err();
    assert!(
        err9.contains("cccp close"),
        "Fase 9 deve orientar o uso obrigatório do comando cccp close"
    );
}
