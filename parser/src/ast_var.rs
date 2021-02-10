
// This file is part of the Lila compiler
// Copyright (C) 2020-2021 Patrick Flynn
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use crate::ast;
use crate::ast::*;
use crate::lex::{Token, Lex};
use crate::syntax::ErrorManager;

use crate::ast_utils::*;

// Builds a variable declaration
pub fn build_var_dec(scanner : &mut Lex, tree : &mut AstTree, current_block : &mut Vec<AstStmt>, name : String, syntax : &mut ErrorManager) -> bool {
    let mut var_dec = ast::create_stmt(AstStmtType::VarDec, scanner);
    var_dec.name = name;
    
    // This will hold any additional names (for multi-variable declarations)
    // If the list is empty, we only have one declaration
    let mut extra_names : Vec<String> = Vec::new();
    
    // Gather information
    // The first token should be the colon, followed by the type and optionally array arguments
    let mut token = scanner.get_token();
    
    while token == Token::Comma {
        token = scanner.get_token();
        
        match token {
            Token::Id(ref val) => extra_names.push(val.to_string()),
            
            _ => {
                syntax.syntax_error(scanner, "Expected variable name.".to_string());
                return false;
            },
        }
        
        token = scanner.get_token();
    }
    
    if token != Token::Colon {
        syntax.syntax_error(scanner, "Expected \':\' after variable name.".to_string());
        return false;
    }
    
    // Now for the type
    let mut is_array = false;
    let mut dtype : DataType;
    let mut sub_type = DataType::None;
    
    token = scanner.get_token();
    
    match token {
        Token::Byte => dtype = DataType::Byte,
        Token::UByte => dtype = DataType::UByte,
        Token::Short => dtype = DataType::Short,
        Token::UShort => dtype = DataType::UShort,
        Token::Int => dtype = DataType::Int,
        Token::UInt => dtype = DataType::UInt,
        Token::Int64 => dtype = DataType::Int64,
        Token::UInt64 => dtype = DataType::UInt64,
        Token::Float => dtype = DataType::Float,
        Token::Double => dtype = DataType::Double,
        Token::Char => dtype = DataType::Char,
        Token::TStr => dtype = DataType::Str,
        
        Token::Id(ref val) => {
            if !ast::enum_exists(tree, val.to_string()) {
                syntax.syntax_error(scanner, "Invalid enumeration.".to_string());
                return false;
            }
            
            dtype = DataType::Enum(val.to_string());
        },
        
        _ => {
            syntax.syntax_error(scanner, "Invalid type.".to_string());
            return false;
        },
    }
    
    // Check for arrays
    token = scanner.get_token();
    
    match token {
        Token::Assign => {},
        
        Token::LBracket => {
            is_array = true;
            if !build_args(scanner, &mut var_dec, Token::RBracket, syntax) {
                return false;
            }
        },
        
        _ => {
            syntax.syntax_error(scanner, "Expected assignment operator.".to_string());
            return false;
        },
    }
    
    // If we have an array, make sure we have the proper syntax and end with the terminator
    // Otherwise, build the assignment
    if is_array {
        sub_type = dtype;
        dtype = DataType::Ptr;
        
        if scanner.get_token() != Token::Semicolon {
            syntax.syntax_error(scanner, "Expected terminator.".to_string());
            return false;
        }
    } else {
        if !build_args(scanner, &mut var_dec, Token::Semicolon, syntax) {
            return false;
        }
        
        var_dec.args = check_operations(&var_dec.args, tree.keep_postfix);
    }
    
    var_dec.data_type = dtype;
    var_dec.sub_type = sub_type;
    current_block.push(var_dec.clone());
    
    for n in extra_names.iter() {
        var_dec.name = n.to_string();
        current_block.push(var_dec.clone());
    }
    
    true
}

// Builds a variable assignment
fn build_var_assign_stmt(scanner : &mut Lex, var_assign : &mut AstStmt, name : String, assign_op : Token, syntax : &mut ErrorManager) -> bool {
    let mut check_end = false;
    
    match assign_op {
        Token::OpInc | Token::OpDec => {
            let mut id_arg = ast::create_arg(AstArgType::Id);
            if var_assign.stmt_type == AstStmtType::ArrayAssign {
                id_arg.sub_args = var_assign.sub_args.clone();
            }
            
            id_arg.str_val = name;
            var_assign.args.push(id_arg);
            
            if assign_op == Token::OpInc {
                let op_arg = ast::create_arg(AstArgType::OpAdd);
                var_assign.args.push(op_arg);
            } else {
                let op_arg = ast::create_arg(AstArgType::OpSub);
                var_assign.args.push(op_arg);
            }
            
            let num_arg = ast::create_int(1);
            var_assign.args.push(num_arg);
            
            check_end = true;
        },
        
        Token::AddAssign | Token::SubAssign 
        | Token::MulAssign | Token::DivAssign 
        | Token::ModAssign => {
            let mut id_arg = ast::create_arg(AstArgType::Id);
            if var_assign.stmt_type == AstStmtType::ArrayAssign {
                id_arg.sub_args = var_assign.sub_args.clone();
            }
            
            id_arg.str_val = name;
            var_assign.args.push(id_arg);
            
            if assign_op == Token::AddAssign {
                let op_arg = ast::create_arg(AstArgType::OpAdd);
                var_assign.args.push(op_arg);
            } else if assign_op == Token::SubAssign {
                let op_arg = ast::create_arg(AstArgType::OpSub);
                var_assign.args.push(op_arg);
            } else if assign_op == Token::MulAssign {
                let op_arg = ast::create_arg(AstArgType::OpMul);
                var_assign.args.push(op_arg);
            } else if assign_op == Token::DivAssign {
                let op_arg = ast::create_arg(AstArgType::OpDiv);
                var_assign.args.push(op_arg);
            } else if assign_op == Token::ModAssign {
                let op_arg = ast::create_arg(AstArgType::OpMod);
                var_assign.args.push(op_arg);
            }
            
            // Build the rest
            if !build_args(scanner, var_assign, Token::Semicolon, syntax) {
                return false;
            }
        },
        
        Token::Assign => {
            if !build_args(scanner, var_assign, Token::Semicolon, syntax) {
                return false;
            }
        },
        
        // TODO: Pls improve this
        _ => {
            syntax.syntax_error(scanner, "Expected \'=\' in array assignment.".to_string());
            return false;
        },
    }
    
    if check_end {
        if scanner.get_token() != Token::Semicolon {
            syntax.syntax_error(scanner, "Expected terminator.".to_string());
            return false;
        }
    }
    
    true
}

// Builds a variable assignment
pub fn build_var_assign(scanner : &mut Lex, current_block : &mut Vec<AstStmt>, name : String, assign_op : Token, keep_postfix : bool, syntax : &mut ErrorManager) -> bool {
    let mut var_assign = ast::create_stmt(AstStmtType::VarAssign, scanner);
    var_assign.name = name.clone();
    
    if !build_var_assign_stmt(scanner, &mut var_assign, name, assign_op, syntax) {
        return false;
    }
    
    var_assign.args = check_operations(&var_assign.args, keep_postfix);
    
    current_block.push(var_assign);
    true
}

// Builds an array assignment
pub fn build_array_assign(scanner : &mut Lex, current_block : &mut Vec<AstStmt>, id_val : String, keep_postfix : bool, syntax : &mut ErrorManager) -> bool {
    let mut array_assign = ast::create_stmt(AstStmtType::ArrayAssign, scanner);
    array_assign.name = id_val.clone();
    
    // For the array index
    if !build_args(scanner, &mut array_assign, Token::RBracket, syntax) {
        return false;
    }
    
    // Build the assignment
    let assign_op = scanner.get_token();
    
    if !build_var_assign_stmt(scanner, &mut array_assign, id_val, assign_op, syntax) {
        return false;
    }
    
    array_assign.args = check_operations(&array_assign.args, keep_postfix);
    
    current_block.push(array_assign);
    
    true
}

// Builds a sizeof operation
pub fn build_sizeof(scanner : &mut Lex, syntax : &mut ErrorManager) -> AstArg {
    let mut sizeof = ast::create_arg(AstArgType::Sizeof);
    
    let token1 = scanner.get_token();   // '('
    let token2 = scanner.get_token();   // ID
    let token3 = scanner.get_token();   // ')'
    
    if token1 != Token::LParen || token3 != Token::RParen {
        syntax.syntax_error(scanner, "Sizeof begins with \'(\' and ends with \')\'".to_string());
        return ast::create_arg(AstArgType::None);
    }
    
    match token2 {
        Token::Id(ref val) => {
            let mut arg = ast::create_arg(AstArgType::Id);
            arg.str_val = val.to_string();
            sizeof.sub_args.push(arg);
        },
        
        _ => {
            syntax.syntax_error(scanner, "Expected variable name.".to_string());
            return ast::create_arg(AstArgType::None);
        },
    }
    
    sizeof
}

// Builds an address-of operation (load the address of a variable)
pub fn build_addrof(scanner : &mut Lex, syntax : &mut ErrorManager) -> AstArg {
    let mut addrof = ast::create_arg(AstArgType::AddrOf);
    let token = scanner.get_token();
    
    match token {
        Token::Id(ref val) => {
            let mut arg = ast::create_arg(AstArgType::Id);
            arg.str_val = val.to_string();
            addrof.sub_args.push(arg);
        },
        
        _ => {
            syntax.syntax_error(scanner, "Expected variable name.".to_string());
            return ast::create_arg(AstArgType::None);
        },
    }
    
    addrof
}

