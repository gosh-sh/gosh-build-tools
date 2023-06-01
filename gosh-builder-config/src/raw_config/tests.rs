use super::*;

#[test]
fn test1() {
    let yaml = r#"
        dockerfile: |-
            FROM ubuntu
    "#;
    let res = RawGoshConfig::try_from(yaml).expect("parse yaml");
    assert_eq!(res.dockerfile, Dockerfile::Content("FROM ubuntu".into()));
    assert_eq!(res.args, None);
}

#[test]
fn test2() {
    let yaml = r#"
        dockerfile: >
            FROM ubuntu
    "#;
    let res = RawGoshConfig::try_from(yaml).expect("parse yaml");
    println!("{:?}", res);
    assert_ne!(res.dockerfile, Dockerfile::Content("FROM ubuntu".into()));
    assert_eq!(res.dockerfile, Dockerfile::Content("FROM ubuntu\n".into()));
    assert_eq!(res.args, None);

    let yaml = serde_yaml::to_string(&res).expect("struct to yaml string");
    assert!(!yaml.contains("args"));
}

#[test]
fn test3() {
    let obj = RawGoshConfig {
        dockerfile: Dockerfile::Content("FROM ubuntu".into()),
        tag: None,
        args: None,
        install: None,
    };

    let yaml = serde_yaml::to_string(&obj).expect("object to yaml conversion");

    let res = RawGoshConfig::try_from(yaml.as_str()).expect("parse yaml");
    assert_eq!(res.dockerfile, Dockerfile::Content("FROM ubuntu".into()));
    assert_eq!(res.args, None);
    assert_eq!(res.tag, None);
    assert_eq!(res.install, None);
}

#[test]
fn test4() {
    let yaml = r#"
        dockerfile:
            path: test/path.yaml
        args:
            arg1: value-2
        tag: test_tag
        install:
            - /test
    "#;
    let res = RawGoshConfig::try_from(yaml).expect("parse yaml");
    assert_eq!(
        res.dockerfile,
        Dockerfile::Path {
            path: "test/path.yaml".to_owned()
        }
    );
    assert_eq!(res.args.unwrap().get("arg1").unwrap(), "value-2");
    assert_eq!(res.tag.unwrap(), "test_tag");
    assert_eq!(res.install.unwrap().first().unwrap(), "/test");
}
