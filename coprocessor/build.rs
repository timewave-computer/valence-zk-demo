use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args(
        "../coprocessor-proofs/coprocessor-circuit-sp1",
        Default::default(),
    );
    build_program_with_args(
        "../zk-programs/zk-rate-example/zk-rate-application",
        Default::default(),
    );
    build_program_with_args(
        "../zk-programs/zk-mailbox-example/zk-mailbox-application",
        Default::default(),
    );
}
