use section_parser_derive::SectionParser;

#[derive(SectionParser)]
struct Test {
    a: Option<String>,
    b: ::std::option::Option<Option<String>>,
}

#[test]
fn ok() {}
