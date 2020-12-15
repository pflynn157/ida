
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::process::Command;

use parser::ltac::{LtacFile, LtacData, LtacDataType, LtacType, LtacInstr, LtacArg};

mod call;
mod func;
mod utils;

use call::*;
use func::*;
use utils::*;

pub fn compile(ltac_file : &LtacFile) -> io::Result<()> {
    let mut name = "/tmp/".to_string();
    name.push_str(&ltac_file.name);
    name.push_str(".asm");
    
    // Write it out
    let file = File::create(&name)?;
    let mut writer = BufWriter::new(file);
    
    write_data(&mut writer, &ltac_file.data);
    write_code(&mut writer, &ltac_file.code);
    
    Ok(())
}

// Assemble a file
pub fn build_asm(name : &String, no_link : bool) {
    // Create all the names
    let mut asm_name = "/tmp/".to_string();
    asm_name.push_str(name);
    asm_name.push_str(".asm");
    
    let mut obj_name = "/tmp/".to_string();
    if no_link {
        obj_name = "./".to_string();
    }
    
    obj_name.push_str(name);
    obj_name.push_str(".o");

    // Assemble
    let asm = Command::new("as")
        .args(&[&asm_name, "-o", &obj_name])
        .output()
        .expect("Fatal: Assembly failed.");
        
    if !asm.status.success() {
        io::stdout().write_all(&asm.stdout).unwrap();
        io::stderr().write_all(&asm.stderr).unwrap();
    }
}
 
// Link everything
pub fn link(all_names : &Vec<String>, output : &String, use_c : bool, is_lib : bool) {
    let mut names : Vec<String> = Vec::new();
    let mut libs : Vec<String> = Vec::new();
    
    for name in all_names.iter() {
        if name.ends_with(".o") {
            names.push(name.clone());
        } else if name.starts_with("-l") {
            libs.push(name.clone());
        } else {
            let mut obj_name = "/tmp/".to_string();
            obj_name.push_str(&name);
            obj_name.push_str(".o");
            names.push(obj_name);
        }
    }
    
    // Link
    //let ld : Output;
    let mut args : Vec<&str> = Vec::new();
    args.push("-L./");
    
    if use_c {
        if !is_lib {
            args.push("/usr/lib64/crti.o");
            args.push("/usr/lib64/crtn.o");
            args.push("/usr/lib64/crt1.o");
        }
        
        args.push("-lc");
    }
    
    args.push("-dynamic-linker");
    args.push("/lib64/ld-linux-riscv64-lp64d.so.1");
        
    for name in names.iter() {
        args.push(&name);
    }
        
    if is_lib {
        args.push("-shared");
    }
    
    for lib in libs.iter() {
        args.push(lib);
    }
        
    args.push("-o");
    args.push(output);
    
    let ld = Command::new("ld")
        .args(args.as_slice())
        .output()
        .expect("Fatal: Linking failed.");
    
    if !ld.status.success() {
        io::stdout().write_all(&ld.stdout).unwrap();
        io::stderr().write_all(&ld.stderr).unwrap();
    }
}

// Write the data section
fn write_data(writer : &mut BufWriter<File>, data : &Vec<LtacData>) {
    let mut line = String::new();
    line.push_str(".data\n");

    for data in data.iter() {
        match &data.data_type {
            LtacDataType::StringL => {
                line.push_str(&data.name);
                line.push_str(": .string \"");
                line.push_str(&data.val);
                line.push_str("\"\n");
            },
            
            LtacDataType::FloatL => {
                line.push_str(&data.name);
                line.push_str(": .long ");
                line.push_str(&data.val);
                line.push_str("\n");
            },
            
            LtacDataType::DoubleL => {
                line.push_str(&data.name);
                line.push_str(": .quad ");
                line.push_str(&data.val);
                line.push_str("\n");
            },
        }
    }
    
    line.push_str("\n");
    
    writer.write(&line.into_bytes())
        .expect("[RISCV64_data] Write failed in .data");
}

// Write the code section
fn write_code(writer : &mut BufWriter<File>, code : &Vec<LtacInstr>) {
    let line = ".text\n".to_string();
    writer.write(&line.into_bytes())
        .expect("[RISCV_code] Write failed");
        
    let mut stack_size = 0;

    for code in code.iter() {
        match &code.instr_type {
        
            // Basic function instructions
            LtacType::Extern => riscv64_build_extern(writer, &code),
            LtacType::Label => riscv64_build_label(writer, &code),
            LtacType::Ret => riscv64_build_ret(writer, stack_size),
            
            LtacType::Func => {
                riscv64_build_func(writer, &code);
                stack_size = code.arg1_val;
            },
            
            // Used to load function arguments
            LtacType::LdArgI8 => {},
            LtacType::LdArgU8 => {},
            LtacType::LdArgI16 => {},
            LtacType::LdArgU16 => {},
            LtacType::LdArgI32 => {},
            LtacType::LdArgU32 => {},
            LtacType::LdArgI64 => {},
            LtacType::LdArgU64 => {},
            LtacType::LdArgF32 => {},
            LtacType::LdArgF64 => {},
            LtacType::LdArgPtr => {},
            
            // All the move instructions
            LtacType::MovB => {},
            LtacType::MovUB => {},
            LtacType::MovW => {},
            LtacType::MovUW => {},
            LtacType::MovU => {},
            LtacType::MovQ => {},
            LtacType::MovUQ => {},
            LtacType::MovF32 => {},
            LtacType::MovF64 => {},
            LtacType::MovI32Vec => {},
            
            // Push/pop
            LtacType::Push => {},
            LtacType::Pop => {},
            
            // Argument and function call instructions
            LtacType::PushArg => riscv64_build_pusharg(writer, &code, false),
            LtacType::KPushArg => riscv64_build_pusharg(writer, &code, true),
            LtacType::Call => riscv64_build_call(writer, &code),
            LtacType::Syscall => {},
            
            // Comparison instructons
            LtacType::I8Cmp => {},
            LtacType::U8Cmp => {},
            LtacType::I16Cmp => {},
            LtacType::U16Cmp => {},
            LtacType::I32Cmp => {},
            LtacType::U32Cmp => {},
            LtacType::I64Cmp => {},
            LtacType::U64Cmp => {},
            LtacType::F32Cmp => {},
            LtacType::F64Cmp => {},
            LtacType::StrCmp => {},
            
            // Branching instructions
            LtacType::Br => {},
            LtacType::Be => {},
            LtacType::Bne => {},
            LtacType::Bl => {},
            LtacType::Ble => {},
            LtacType::Bfl => {},
            LtacType::Bfle => {},
            LtacType::Bg => {},
            LtacType::Bge => {},
            LtacType::Bfg => {},
            LtacType::Bfge => {},
            
            // Signed byte math operations
            LtacType::I8Add => {},
            LtacType::I8Sub => {},
            LtacType::I8Mul => {},
            LtacType::I8Div => {},
            LtacType::I8Mod => {},
            
            // Unsigned byte math operations
            LtacType::U8Add => {},
            LtacType::U8Mul => {},
            LtacType::U8Div => {},
            LtacType::U8Mod => {},
            
            // Signed word (2-byte) math operations
            LtacType::I16Add => {},
            LtacType::I16Sub => {},
            LtacType::I16Mul => {},
            LtacType::I16Div => {},
            LtacType::I16Mod => {},
            
            // Unsigned word (2-byte) math operations
            LtacType::U16Add => {},
            LtacType::U16Mul => {},
            LtacType::U16Div => {},
            LtacType::U16Mod => {},
            
            // Byte bitwise operations
            LtacType::BAnd => {},
            LtacType::BOr => {},
            LtacType::BXor => {},
            LtacType::BLsh => {},
            LtacType::BRsh => {},
            
            // Word bitwise operations
            LtacType::WAnd => {},
            LtacType::WOr => {},
            LtacType::WXor => {},
            LtacType::WLsh => {},
            LtacType::WRsh => {},
            
            // Signed 32-bit integer math opreations
            LtacType::I32Add => {},
            LtacType::I32Sub => {},
            LtacType::I32Mul => {},
            LtacType::I32Div => {},
            LtacType::I32Mod => {},
            
            // Unsigned 32-bit integer math opreations
            LtacType::U32Add => {},
            LtacType::U32Mul => {},
            LtacType::U32Div => {},
            LtacType::U32Mod => {},
            
            // Signed 32-bit integer bitwise operations
            LtacType::I32And => {},
            LtacType::I32Or => {},
            LtacType::I32Xor => {},
            LtacType::I32Lsh => {},
            LtacType::I32Rsh => {},
            
            // Signed 32-bit vector math operations
            LtacType::I32VAdd => {},
            
            // Signed 64-bit integer math operations
            LtacType::I64Add => {},
            LtacType::I64Sub => {},
            LtacType::I64Mul => {},
            LtacType::I64Div => {},
            LtacType::I64Mod => {},
            
            // Unsigned 64-bit integer math operations
            LtacType::U64Add => {},
            LtacType::U64Mul => {},
            LtacType::U64Div => {},
            LtacType::U64Mod => {},
            
            // 64-bit integer bitwise operations
            LtacType::I64And => {},
            LtacType::I64Or => {},
            LtacType::I64Xor => {},
            LtacType::I64Lsh => {},
            LtacType::I64Rsh => {},
            
            // Single-precision float operations
            LtacType::F32Add => {},
            LtacType::F32Sub => {},
            LtacType::F32Mul => {},
            LtacType::F32Div => {},
            
            // Double-precision float operations
            LtacType::F64Add => {},
            LtacType::F64Sub => {},
            LtacType::F64Mul => {},
            LtacType::F64Div => {},
            
            // These are intrinsics if you will; they should never get down to a code generation layer
            LtacType::Exit => {},
            LtacType::Malloc => {},
            LtacType::Free => {},
            
            // These are specific to RISC machines
            // RISC Load instructions
            LtacType::LdB => {},
            LtacType::LdUB => {},
            LtacType::LdW => {},
            LtacType::LdUW => {},
            LtacType::Ld => riscv64_build_ld_str(writer, &code),
            LtacType::LdU => {},
            LtacType::LdQ => {},
            LtacType::LdUQ => {},
            LtacType::LdF32 => {},
            LtacType::LdF64 => {},
            
            // RISC store instructions
            LtacType::StrB => {},
            LtacType::StrUB => {},
            LtacType::StrW => {},
            LtacType::StrUW => {},
            LtacType::Str => riscv64_build_ld_str(writer, &code),
            LtacType::StrU => {},
            LtacType::StrQ => {},
            LtacType::StrUQ => {},
            LtacType::StrF32 => {},
            LtacType::StrF64 => {},
            LtacType::StrPtr => {},
            
            // All else
            _ => riscv64_build_instr(writer, &code),
        }
    }
}

// Builds the load-store instructions
fn riscv64_build_ld_str(writer : &mut BufWriter<File>, code : &LtacInstr) {
    let mut line = String::new();

    match &code.instr_type {
        LtacType::Ld => line.push_str("  ldr "),
        LtacType::Str => line.push_str("  str "),

        _ => {},
    }

    // Write the registers
    match &code.arg2 {
        LtacArg::Reg32(pos) => {
            let reg = riscv64_op_reg32(*pos);
            line.push_str(&reg);
        },

        _ => {},
    }

    line.push_str(", ");

    // Write out the memory
    match &code.arg1 {
        LtacArg::Mem(pos) => {
            line.push_str("[sp, ");
            line.push_str(&pos.to_string());
            line.push_str("]");
        },

        _ => {},
    }

    // Write the rest out
    line.push_str("\n");

    writer.write(&line.into_bytes())
        .expect("[RISCV64_build_ld_str] Write failed.");
}

// For RISC-V instructions that have a common syntax
fn riscv64_build_instr(writer : &mut BufWriter<File>, code : &LtacInstr) {
    let mut line = String::new();

    // Write the instruction type
    match &code.instr_type {
        LtacType::Mov => {
            match &code.arg2 {
                LtacArg::I32(_v) => line.push_str("  li "),
                _ => line.push_str("  mv "),
            }
        },
        
        _ => {},
    }

    // Write the first operand
    match &code.arg1 {
        LtacArg::RetRegI32 | LtacArg::RetRegU32 => line.push_str("a0, "),

        LtacArg::Reg32(pos) => {
            let reg = riscv64_op_reg32(*pos);
            line.push_str(&reg);
            line.push_str(", ");
        },
        
        _ => {},
    }

    // Write the second operand
    match &code.arg2 {
        LtacArg::I32(val) => line.push_str(&val.to_string()),

        _ => {},
    }

    // Finish writing
    line.push_str("\n");

    writer.write(&line.into_bytes())
        .expect("[RISCV64_write_instr] Write failed.");
}