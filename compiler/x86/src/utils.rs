
// Gets a register based on position
// Kernel argument registers
pub fn amd64_karg_reg32(pos : i32) -> String {
    match pos {
        1 => return "eax".to_string(),
        2 => return "edi".to_string(),
        3 => return "esi".to_string(),
        4 => return "edx".to_string(),
        _ => return String::new(),
    };
}

pub fn amd64_karg_reg64(pos : i32) -> String {
    match pos {
        1 => return "rax".to_string(),
        2 => return "rdi".to_string(),
        3 => return "rsi".to_string(),
        4 => return "rdx".to_string(),
        _ => return String::new(),
    };
}

// Function argument registers
pub fn amd64_arg_reg32(pos : i32) -> String {
    match pos {
        1 => return "edi".to_string(),
        2 => return "esi".to_string(),
        3 => return "edx".to_string(),
        4 => return "ecx".to_string(),
        5 => return "r8d".to_string(),
        6 => return "r9d".to_string(),
        _ => return String::new(),
    };
}

pub fn amd64_arg_reg64(pos : i32) -> String {
    match pos {
        1 => return "rdi".to_string(),
        2 => return "rsi".to_string(),
        3 => return "rdx".to_string(),
        4 => return "rcx".to_string(),
        5 => return "r8".to_string(),
        6 => return "r9".to_string(),
        _ => return String::new(),
    };
}

// Operation registers
// EAX -> Return register
// R15, R14 -> Operations register
pub fn amd64_op_reg32(pos : i32) -> String {
    match pos {
        0 => return "ebx".to_string(),
        1 => return "edx".to_string(),
        2 => return "r10d".to_string(),
        _ => return String::new(),
    };
}

// Vector registers
// ymm0 and ymm1 are reserved for internal operations
pub fn amd64_vector_i32(pos : i32) -> String {
    match pos {
        0 => return "ymm2".to_string(),
        1 => return "ymm3".to_string(),
        2 => return "ymm4".to_string(),
        3 => return "ymm5".to_string(),
        4 => return "ymm6".to_string(),
        5 => return "ymm7".to_string(),
        _ => return String::new(),
    }
}
