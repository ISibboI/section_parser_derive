use section_parser_derive::SectionParser;

#[derive(Debug, Eq, PartialEq)]
struct UnderlyingType<T> {
    t: T,
    abc: usize,
}

#[derive(SectionParser)]
struct Test {
    a: Option<UnderlyingType<String>>,
    b: Option<UnderlyingType<Option<String>>>,
}

#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum TestError {
    MissingField(String),
    DuplicateField(String),
    UnexpectedField(String),
}

impl Test {
    fn missing_field_error(&self, field: &str) -> TestError {
        TestError::MissingField(field.to_string())
    }

    fn duplicate_field_error<T>(&self, field: &str, _value: UnderlyingType<T>) -> TestError {
        TestError::DuplicateField(field.to_string())
    }

    fn unexpected_field_error<T>(&self, field: &str, _value: UnderlyingType<T>) -> TestError {
        TestError::UnexpectedField(field.to_string())
    }
}

#[test]
fn ok() {
    let mut test = Test {
        a: Some(UnderlyingType {
            t: "acb".to_string(),
            abc: 2,
        }),
        b: None,
    };

    test.a().unwrap();
    test.set_b(UnderlyingType { t: None, abc: 3 }).unwrap();
    assert_eq!(test.b(), Ok(UnderlyingType { t: None, abc: 3 }));
    test.ensure_empty().unwrap();
}
