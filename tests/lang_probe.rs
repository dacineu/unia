#[test]
fn probe_multilingual() {
    use unia::session::{lessons, summarise, Turn};
    let t = |s: &str, x: &str| Turn { speaker: s.into(), text: x.into() };

    // A Romanian session where the person corrects the model three times.
    let ro = vec![
        t("human", "adauga un comentariu la parser"),
        t("assistant", "Am adaugat `/// Parseaza o afirmatie` deasupra functiei."),
        t("human", "nu, comentariul trebuie sa descrie valoarea returnata"),
        t("assistant", "Am rescris comentariul."),
        t("human", "in loc de asta foloseste o referinta"),
        t("assistant", "Am schimbat."),
        t("human", "nu, mai bine sa fie o valoare, nu o referinta"),
    ];
    let s = summarise(&ro);
    println!("Romanian session: {s:?}");
    println!("  lessons: {}", lessons(&ro).len());

    // Same session in English, for comparison.
    let en = vec![
        t("human", "add a comment to the parser"),
        t("assistant", "Added `/// Parses a statement`."),
        t("human", "no, the comment should describe the return value"),
        t("assistant", "Rewritten."),
        t("human", "instead use a reference"),
        t("assistant", "Changed."),
        t("human", "no, better as a value than a reference"),
    ];
    println!("English session:   {:?}", summarise(&en));
    println!("  lessons: {}", lessons(&en).len());
}
