
// This file is part of the Lila compiler
// Copyright (C) 2020 Patrick Flynn
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
pub fn build_var_dec(scanner : &mut Lex, tree : &mut AstTree, name : String, syntax : &mut ErrorManager) -> bool {
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
    let dtype : AstModType;
    
    token = scanner.get_token();
    
    match token {
        Token::Byte => dtype = AstModType::Byte,
        Token::UByte => dtype = AstModType::UByte,
        Token::Short => dtype = AstModType::Short,
        Token::UShort => dtype = AstModType::UShort,
        Token::Int => dtype = AstModType::Int,
        Token::UInt => dtype = AstModType::UInt,
        Token::Int64 => dtype = AstModType::Int64,
        Token::UInt64 => dtype = AstModType::UInt64,
        Token::Float => dtype = AstModType::Float,
        Token::Double => dtype = AstModType::Double,
        Token::Char => dtype = AstModType::Char,
        Token::TStr => dtype = AstModType::Str,
        
        Token::Id(ref val) => {
            if !ast::enum_exists(tree, val.to_string()) {
                syntax.syntax_error(scanner, "Invalid enumeration.".to_string());
                return false;
            }
            
            dtype = AstModType::Enum(val.to_string());
        },
        
        _ => {
            syntax.syntax_error(scanner, "Invalid type.".to_string());
            return false;
        },
    }
        
    let mut data_type = AstMod {
        mod_type : dtype.clone(),
    };
    
    // Check for arrays
    token = scanner.get_token();
    
    match token {
        Token::Assign => {},
        
        Token::LBracket => {
            is_array = true;
            if !build_args(scanner, &mut var_dec, Token::RBracket, syntax) {
                return false;
            }
            
            token = scanner.get_token();
            
            if token != Token::Assign {
                syntax.syntax_error(scanner, "Expected assignment operator.".to_string());
                return false;
            }
        },
        
        _ => {
            syntax.syntax_error(scanner, "Expected assignment operator.".to_string());
            return false;
        },
    }
    
    // Build the remaining arguments
    if !build_args(scanner, &mut var_dec, Token::Semicolon, syntax) {
        return false;
    }
    
    var_dec.args = check_operations(&var_dec.args);
    
    // If we have the array, check the array type
    if is_array {
        if var_dec.args.len() == 1 && var_dec.args.last().unwrap().arg_type == AstArgType::Array {
            match &dtype {
                AstModType::Byte | AstModType::Char => data_type.mod_type = AstModType::ByteDynArray,
                AstModType::UByte => data_type.mod_type = AstModType::UByteDynArray,
                AstModType::Short => data_type.mod_type = AstModType::ShortDynArray,
                AstModType::UShort => data_type.mod_type = AstModType::UShortDynArray,
                AstModType::Int => data_type.mod_type = AstModType::IntDynArray,
                AstModType::UInt => data_type.mod_type = AstModType::UIntDynArray,
                AstModType::Int64 => data_type.mod_type = AstModType::I64DynArray,
                AstModType::UInt64 => data_type.mod_type = AstModType::U64DynArray,
                AstModType::Float => data_type.mod_type = AstModType::FloatDynArray,
                AstModType::Double => data_type.mod_type = AstModType::DoubleDynArray,
                AstModType::Str => data_type.mod_type = AstModType::StrDynArray,
                
                _ => {},
            }
        } else {
            //TODO
        }
    }
    
    var_dec.modifiers.push(data_type);
    ast::add_stmt(tree, var_dec.clone());
    
    for n in extra_names.iter() {
        var_dec.name = n.to_string();
        ast::add_stmt(tree, var_dec.clone());
    }
    
    true
}

// Builds a variable assignment
pub fn build_var_assign(scanner : &mut Lex, tree : &mut AstTree, name : String, syntax : &mut ErrorManager) -> bool {
    let mut var_assign = ast::create_stmt(AstStmtType::VarAssign, scanner);
    var_assign.name = name;
    
    if !build_args(scanner, &mut var_assign, Token::Semicolon, syntax) {
        return false;
    }
    
    ast::add_stmt(tree, var_assign);
    
    true
}

// Builds an array assignment
pub fn build_array_assign(scanner : &mut Lex, tree : &mut AstTree, id_val : String, syntax : &mut ErrorManager) -> bool {
    let mut array_assign = ast::create_stmt(AstStmtType::ArrayAssign, scanner);
    array_assign.name = id_val;
    
    // For the array index
    if !build_args(scanner, &mut array_assign, Token::RBracket, syntax) {
        return false;
    }
    
    if scanner.get_token() != Token::Assign {
        syntax.syntax_error(scanner, "Expected \'=\' in array assignment.".to_string());
        return false;
    }
    
    // Tokens being assigned to the array
    if !build_args(scanner, &mut array_assign, Token::Semicolon, syntax) {
        return false;
    }
    
    ast::add_stmt(tree, array_assign);
    
    true
}

