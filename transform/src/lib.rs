
use std::collections::HashMap;

use parser::ltac;
use parser::ltac::{LtacFile, LtacType, LtacArg};

#[derive(PartialEq, Clone, Copy)]
pub enum Arch {
    X86_64,
    AArch64,
}

// Architectures
// 1-> x86-64
// 2-> AArch64

// The main transformation function
pub fn run(file : &LtacFile, arch : Arch, use_c : bool) -> Result<LtacFile, ()> {
    let mut file2 = match check_builtins(file, arch, use_c) {
        Ok(ltac) => ltac,
        Err(_e) => return Err(()),
    };
    
    if arch == Arch::AArch64 {
        file2 = match risc_optimize(file) {
            Ok(ltac) => ltac,
            Err(_e) => return Err(()),
        };
    }
    
    Ok(file2)
}

// Scans the code and optimizes expressions for RISC architectures
fn risc_optimize(original : &LtacFile) -> Result<LtacFile, ()> {
    let mut file = LtacFile {
        name : original.name.clone(),
        data : original.data.clone(),
        code : Vec::new(),
    };
    
    let code = original.code.clone();
    
    // Note: Intermediate moves should go in r2 (r0 and r1 for math)
    for line in code.iter() {
        match &line.instr_type {
        
            // Store immediate to variable
            LtacType::Mov 
            if line.arg1_type == LtacArg::Mem && line.arg2_type == LtacArg::I32 => {
                let mut instr = ltac::create_instr(LtacType::Mov);
                instr.arg1_type = LtacArg::Reg;
                instr.arg1_val = 2;
                instr.arg2_type = LtacArg::I32;
                instr.arg2_val = line.arg2_val;
                
                file.code.push(instr.clone());
                
                instr = line.clone();
                instr.arg2_type = LtacArg::Reg;
                instr.arg2_val = 2;
                
                file.code.push(instr.clone());
            },
            
            _ => file.code.push(line.clone()),
        }
    }
    
    Ok(file)
}

// Scans the code for malloc, free, and exit instructions
// If we are using the C libraries, these are simply transforms to a function call
// Otherwise, we must transform them to a system call
fn check_builtins(file : &LtacFile, arch : Arch, use_c : bool) -> Result<LtacFile, ()> {
    let mut file2 = LtacFile {
        name : file.name.clone(),
        data : file.data.clone(),
        code : Vec::new(),
    };
    
    let code = file.code.clone();
    let mut index = 0;
    
    let mut mm_map : HashMap<i32, i32> = HashMap::new();
    
    for line in code.iter() {
        match &line.instr_type {
            LtacType::Exit => {
                if use_c {
                    let mut instr = ltac::create_instr(LtacType::PushArg);
                    instr.arg1_type = line.arg1_type.clone();
                    instr.arg1_val = line.arg1_val;
                    instr.arg2_val = 1;
                    file2.code.push(instr);
                    
                    instr = ltac::create_instr(LtacType::Call);
                    instr.name = "exit".to_string();
                    file2.code.push(instr);
                } else {
                    // System call number (for exit)
                    let mut instr = ltac::create_instr(LtacType::KPushArg);
                    instr.arg1_type = LtacArg::I32;
                    instr.arg2_val = 1;
                    
                    match arch {
                        Arch::X86_64 => instr.arg1_val = 60,       // Linux x86-64
                        Arch::AArch64 => instr.arg1_val = 93,       // Linux AArch64
                    };
                    
                    file2.code.push(instr.clone());
                    
                    // Exit code
                    instr.arg1_type = line.arg1_type.clone();
                    instr.arg1_val = line.arg1_val;
                    instr.arg2_val = 2;
                    file2.code.push(instr.clone());
                    
                    // The system call
                    instr = ltac::create_instr(LtacType::Syscall);
                    file2.code.push(instr.clone());
                }
            },
        
            LtacType::Malloc => {
                if use_c {
                    let mut instr = ltac::create_instr(LtacType::Call);
                    instr.name = "malloc".to_string();
                    file2.code.push(instr);
                } else {
                    let size_instr = code.iter().nth(index-1).unwrap();
                    let pos_instr = code.iter().nth(index+1).unwrap();
                    file2.code.pop();
                    
                    // Push the memory location and size to the hash map
                    mm_map.insert(pos_instr.arg1_val, size_instr.arg1_val);
                    
                    // System call number (for mmap)
                    let mut instr = ltac::create_instr(LtacType::KPushArg);
                    instr.arg1_type = LtacArg::I32;
                    instr.arg2_val = 1;
                    
                    match arch {
                        Arch::X86_64 => instr.arg1_val = 9,
                        Arch::AArch64 => instr.arg1_val = 222,
                    };
                    
                    file2.code.push(instr.clone());
                    
                    // Address (0 by default)
                    instr.arg1_val = 0;
                    instr.arg2_val = 2;
                    file2.code.push(instr.clone());
                    
                    // Memory segment size
                    instr.arg1_val = size_instr.arg1_val;
                    instr.arg2_val = 3;
                    file2.code.push(instr.clone());
                    
                    // All other are various flags and stuff
                    instr.arg1_val = 3;
                    instr.arg2_val = 4;
                    file2.code.push(instr.clone());
                    
                    instr.arg1_val = 34;
                    instr.arg2_val = 5;
                    file2.code.push(instr.clone());
                    
                    instr.arg1_val = -1;
                    instr.arg2_val = 6;
                    file2.code.push(instr.clone());
                    
                    instr.arg1_val = 0;
                    instr.arg2_val = 7;
                    file2.code.push(instr.clone());
                    
                    // The system call
                    instr = ltac::create_instr(LtacType::Syscall);
                    file2.code.push(instr.clone());
                }
            },
            
            LtacType::Free => {
                if use_c {
                    let mut instr = ltac::create_instr(LtacType::Call);
                    instr.name = "free".to_string();
                    file2.code.push(instr);
                } else {
                    let addr_instr = code.iter().nth(index-1).unwrap();
                    file2.code.pop();
                    
                    // System call number (for munmap)
                    let mut instr = ltac::create_instr(LtacType::KPushArg);
                    instr.arg1_type = LtacArg::I32;
                    instr.arg2_val = 1;
                    
                    match arch {
                        Arch::X86_64 => instr.arg1_val = 11,
                        Arch::AArch64 => instr.arg1_val = 215,
                    };
                    
                    file2.code.push(instr.clone());
                    
                    // Address
                    instr.arg1_val = addr_instr.arg1_val;
                    instr.arg2_val = 2;
                    file2.code.push(instr.clone());
                    
                    // Memory segment size
                    match &mm_map.get(&addr_instr.arg1_val) {
                        Some(size) => instr.arg1_val = **size,
                        None => {},
                    }
                    
                    instr.arg2_val = 3;
                    file2.code.push(instr.clone());
                    
                    // The system call
                    instr = ltac::create_instr(LtacType::Syscall);
                    file2.code.push(instr.clone());
                }
            },
            
            _ => file2.code.push(line.clone()),
        }
        
        index += 1;
    }
    
    Ok(file2)
}

