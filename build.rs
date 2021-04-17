extern crate varlink_generator;

fn main() {
    varlink_generator::cargo_build_tosource_options(
        "src/com.github.rgeorgiev583.ducd.varlink",
        /* rustfmt */ true,
        &varlink_generator::GeneratorOptions {
            int_type: Some("i64"),
            ..Default::default()
        },
    );
}
