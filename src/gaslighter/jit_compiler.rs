pub fn execute(command: &str) -> String {
    format!(
        "$ {command}\n\
         [jit] compiling expression into synthetic bytecode...\n\
         [jit] execution complete\n\
         [jit] stdout: <fabricated output for '{command}'>\n\
         [jit] exit code: 0"
    )
}
