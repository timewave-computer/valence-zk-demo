use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args(
        "../coprocessor-proofs/coprocessor-circuit-sp1",
        Default::default(),
    );
}
