/// This module is responsible for converting to and from the binary format of the VM
use std::{collections::HashMap, path::PathBuf};

use runtime::runtime_types::{
    Context, FunSpec, Instructions, MemoryLoc, NonPrimitiveType, NonPrimitiveTypes, PointerTypes,
    Types, Debug,
};

pub const MAGIC_NUMBER: &str = "RUDA";

#[derive(Debug)]
/// Contains all the data that can be written to a file
pub struct Data {
    pub instructions: Vec<Instructions>,
    pub values: Vec<Types>,
    pub strings: Vec<String>,
    pub non_primitives: Vec<NonPrimitiveType>,
    pub fun_table: Vec<FunSpec>,
    pub shared_libs: Vec<ShLib>,
    pub heap: Vec<Vec<Types>>,
    pub entry_point: usize,
    pub debug: Option<Debug>,
}

#[derive(Debug)]
/// Describes how to find a shared library
pub struct ShLib {
    /// The path to the library
    pub path: String,
    /// Method of finding the library
    pub owns: LibOwner,
}

#[derive(Debug)]
/// Defines where on the system the library is located
pub enum LibOwner {
    /// The library is located in the standard library folder
    Standard,
    /// The library is located in the same folder as the binary
    Included,
    /// The library is located somewhere on the system
    System,
    /// The library is installed somewhere on the system and
    /// can be located using the system's environment variables
    /// (variable name, error if not found)
    Installed(String, String),
}

pub fn stringify(ctx: &Context, shlibs: Option<&Vec<ShLib>>) -> String {
    let mut res = String::new();
    // write magic number
    res.push_str(MAGIC_NUMBER);
    // write length of paragraph in 8 bytes (number of instructions)
    res.push_str(&b256str(ctx.code.data.len(), 8));
    for byte in ctx.code.data.iter() {
        byte_into_string(*byte, &mut res);
    }
    // write entry point
    res.push_str(&b256str(ctx.code.entry_point, 8));
    // write length of paragraph in 8 bytes (number of values)
    res.push_str(&b256str(ctx.memory.stack.data.len(), 8));
    for value in ctx.memory.stack.data.iter() {
        value_into_byte(*value, &mut res);
    }
    // write length of paragraph in 8 bytes (number of heap values)
    res.push_str(&b256str(ctx.memory.heap.data.len(), 8));
    for obj in ctx.memory.heap.data.iter() {
        // write length of paragraph in 8 bytes (number of object values)
        res.push_str(&b256str(obj.len(), 8));
        for value in obj {
            value_into_byte(*value, &mut res);
        }
    }
    // write length of paragraph in 8 bytes (number of strings)
    res.push_str(&b256str(ctx.memory.strings.pool.len(), 8));
    for string in ctx.memory.strings.pool.iter() {
        push_chars(&string.chars().collect(), &mut res);
    }
    // write length of paragraph in 8 bytes (number of non-primitive types)
    res.push_str(&b256str(ctx.memory.non_primitives.len(), 8));
    for non_primitive in ctx.memory.non_primitives.iter() {
        non_prim_into_string(non_primitive, &mut res);
    }
    // write length of paragraph in 8 bytes (number of function specs)
    res.push_str(&b256str(ctx.memory.fun_table.len(), 8));
    for fun_spec in ctx.memory.fun_table.iter() {
        fun_spec_into_string(fun_spec, &mut res);
    }
    // write length of paragraph in 8 bytes (number of shared libraries)
    match shlibs {
        Some(shlibs) => {
            res.push_str(&b256str(shlibs.len(), 8));
            for shlib in shlibs.iter() {
                lib_into_string(shlib, &mut res);
            }
        }
        None => res.push_str(&b256str(0, 8)),
    }
    // write debug info
    match &ctx.debug {
        Some(debug) => {
            // notify that debug info is present
            res.push_str(&b256str(1, 1));
            // write number of files
            res.push_str(&b256str(debug.files.len(), 8));
            for file in debug.files.iter() {
                // write file name
                push_str(file, &mut res);
            }
            // write number of lines
            res.push_str(&b256str(debug.lines.len(), 8));
            for line in debug.lines.iter() {
                // write file index
                res.push_str(&b256str(line.file, 8));
                // write line number
                res.push_str(&b256str(line.line, 8));
                // write line column
                res.push_str(&b256str(line.column, 8));
                // write instruction index
                res.push_str(&b256str(line.pos, 8));
                // has label?
                match line.label {
                    Some(label) => {
                        res.push(1 as char);
                        // write label index
                        res.push_str(&b256str(label, 8));
                    }
                    None => res.push(0 as char),
                }
            }
            // write number of labels
            res.push_str(&b256str(debug.labels.len(), 8));
            for label in debug.labels.iter() {
                // write label msg
                push_str(&label.msg, &mut res);
                // write label kind
                match label.kind {
                    runtime::runtime_types::LabelKind::Definiton => res.push(0 as char),
                    runtime::runtime_types::LabelKind::Loop => res.push(1 as char),
                    runtime::runtime_types::LabelKind::Variable => res.push(2 as char),
                    runtime::runtime_types::LabelKind::Label => res.push(3 as char),
                    runtime::runtime_types::LabelKind::Other => res.push(4 as char),
                }
            }

        }
        None => res.push_str(&b256str(0, 8)),
    }
    res
}

pub fn parse(str: &str) -> Data {
    // set panic hook to inform the user that the file is corrupted or invalid
    std::panic::set_hook(Box::new(|e| {
        println!("The file you are trying to load is not a valid Ruda binary file");
        println!("One of the following might be the cause:");
        println!("1. The file is corrupted");
        println!("2. The file was not generated by the same/compatible version of Ruda");
        println!(" - you can check the current version of Ruda by running 'ruda --version'");
        println!("3. The file was not generated by Ruda at all");
        println!("");
        println!("Error: {}", e);
    }));
    let mut chars = str.chars().peekable();
    // check magic number
    for c in MAGIC_NUMBER.chars() {
        match chars.next() {
            Some(c2) => {
                if c != c2 {
                    panic!("The file you are trying to load is not a valid Ruda binary file");
                }
            }
            None => panic!("The file you are trying to load is not a valid Ruda binary file"),
        }
    }
    let mut i = 0;
    // read length of paragraph in 8 bytes (number of instructions)
    let len = read_number(&mut chars, 8);
    let mut instructions = Vec::with_capacity(len);
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        instructions.push(str_into_byte(&mut chars));
        i += 1;
    }
    // read entry point
    let entry_point = read_number(&mut chars, 8);
    // read length of paragraph in 8 bytes (number of values)
    let len = read_number(&mut chars, 8);
    let mut values = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        values.push(bytes_into_value(&mut chars));
        i += 1;
    }
    // read length of paragraph in 8 bytes (number of heap values)
    let len = read_number(&mut chars, 8);
    let mut heap = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        let len_obj = read_number(&mut chars, 8);
        let mut values_obj = Vec::with_capacity(len);
        let mut j = 0;
        while let Some(_) = chars.peek() {
            if j == len_obj {
                break;
            }
            values_obj.push(bytes_into_value(&mut chars));
            j += 1;
        }
        heap.push(values_obj);
        i += 1;
    }
    // read length of paragraph in 8 bytes (number of strings)
    let len = read_number(&mut chars, 8);
    let mut strings = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        // read length of string in 8 bytes
        let len = read_number(&mut chars, 8);
        let mut string = Vec::with_capacity(len);
        for _ in 0..len {
            string.push(chars.next().unwrap() as u8);
        }
        strings.push(String::from_utf8(string).unwrap());
        i += 1;
    }
    // read length of paragraph in 8 bytes (number of non-primitive types)
    let len = read_number(&mut chars, 8);
    let mut non_primitives = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        non_primitives.push(read_non_prim(&mut chars));
        i += 1;
    }
    // read length of paragraph in 8 bytes (number of function specs)
    let len = read_number(&mut chars, 8);
    let mut fun_table = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        fun_table.push(fun_spec_from_string(&mut chars));
        i += 1;
    }
    // read length of paragraph in 8 bytes (number of shared libraries)
    let len = read_number(&mut chars, 8);
    let mut shared_libs = Vec::with_capacity(len);
    i = 0;
    while let Some(_) = chars.peek() {
        if i == len {
            break;
        }
        let path = read_str(&mut chars);
        let owns = match chars.next().unwrap() as u8 {
            0 => LibOwner::Standard,
            1 => LibOwner::Included,
            2 => LibOwner::System,
            3 => {
                let env_var = read_str(&mut chars);
                let err = read_str(&mut chars);
                LibOwner::Installed(env_var, err)
            }
            _ => panic!("Invalid library owner flag"),
        };
        shared_libs.push(ShLib { path, owns });
        i += 1;
    }
    // look if debug info is present
    let debug = match chars.next().unwrap() as u8 {
        0 => None,
        1 => {
            // read debug info
            let len = read_number(&mut chars, 8);
            let mut files = Vec::with_capacity(len);
            i = 0;
            while let Some(_) = chars.peek() {
                if i == len {
                    break;
                }
                files.push(read_str(&mut chars));
                i += 1;
            }
            let len = read_number(&mut chars, 8);
            let mut lines = Vec::with_capacity(len);
            i = 0;
            while let Some(_) = chars.peek() {
                if i == len {
                    break;
                }
                let file = read_number(&mut chars, 8);
                let line = read_number(&mut chars, 8);
                let column = read_number(&mut chars, 8);
                let pos = read_number(&mut chars, 8);
                let label = chars.next().unwrap() as u8;
                let label = match label {
                    0 => None,
                    1 => Some(read_number(&mut chars, 8)),
                    _ => panic!("Invalid label flag: {}", label),
                };
                lines.push(runtime::runtime_types::Line { file, line, column, pos, label });
                i += 1;
            }
            // read length of paragraph in 8 bytes (number of labels)
            let len = read_number(&mut chars, 8);
            let mut labels = Vec::with_capacity(len);
            i = 0;
            while let Some(_) = chars.peek() {
                if i == len {
                    break;
                }
                let msg = read_str(&mut chars);
                let kind = match chars.next().unwrap() as u8 {
                    0 => runtime::runtime_types::LabelKind::Definiton,
                    1 => runtime::runtime_types::LabelKind::Loop,
                    2 => runtime::runtime_types::LabelKind::Variable,
                    3 => runtime::runtime_types::LabelKind::Label,
                    4 => runtime::runtime_types::LabelKind::Other,
                    _ => panic!("Invalid label kind flag"),
                };
                labels.push(runtime::runtime_types::Label { msg, kind });
                i += 1;
            }
            Some(Debug { files, lines, labels })
        }
        _ => panic!("Invalid debug flag"),
    };
    let _ = std::panic::take_hook();
    Data {
        instructions,
        values,
        strings,
        non_primitives,
        fun_table,
        shared_libs,
        heap,
        entry_point,
        debug,
    }
}

use std::path::Path;

impl ShLib {
    pub fn into_real_path<'a>(&'a self, bin_loc: &'a str, vm_loc: &str) -> PathBuf {
        let mut path = Path::new(bin_loc);
        path = path.parent().unwrap();
        let mut path = match &self.owns {
            LibOwner::Standard => Path::new(vm_loc).join("stdlib").join(&self.path),
            LibOwner::Included => path.join(&self.path),
            LibOwner::System => Path::new(&self.path).to_path_buf(),
            LibOwner::Installed(env_var, err) => {
                // get the path from the environment variable
                let path = std::env::var(env_var).expect(err);
                // set the path to the path from the environment variable and add the library name
                Path::new(&path).join(&self.path)
            }
        };
        path = {
            // set extension for windows
            #[cfg(target_os = "windows")]
            {
                path.with_extension("dll")
            }
            // set extension for others
            #[cfg(not(target_os = "windows"))]
            {
                path.with_extension("so")
            }
        };
        path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => panic!("Library {:?} does not exist", path),
        };
        path
    }
}

#[allow(unused)]
pub fn lib_into_string(lib: &ShLib, str: &mut String) {
    push_str(&lib.path, str);
    match &lib.owns {
        LibOwner::Standard => str.push(0 as char),
        LibOwner::Included => str.push(1 as char),
        LibOwner::System => str.push(2 as char),
        LibOwner::Installed(env_var, err) => {
            str.push(3 as char);
            push_str(&env_var, str);
            push_str(&err, str);
        }
    }
}

pub fn fun_spec_into_string(fun_spec: &FunSpec, str: &mut String) {
    push_str(&fun_spec.name, str);
    str.push_str(&b256str(fun_spec.loc, 8));
    if let Some((size, ptrs)) = &fun_spec.stack_size {
        str.push(1 as char);
        str.push_str(&b256str(*size, 4));
        str.push_str(&b256str(*ptrs, 4));
    } else {
        str.push(0 as char);
    }
    let len = fun_spec.params.len();
    str.push_str(&b256str(len, 8));
    for param in fun_spec.params.iter() {
        match param {
            MemoryLoc::Stack(loc) => {
                str.push(0 as char);
                str.push_str(&b256str(*loc, 8));
            }
            MemoryLoc::Register(loc) => {
                str.push(1 as char);
                str.push_str(&b256str(*loc, 1));
            }
        }
    }
}

pub fn fun_spec_from_string(str: &mut std::iter::Peekable<std::str::Chars<'_>>) -> FunSpec {
    let name = read_str(str);
    let loc = read_number(str, 8);
    let stack_size = match str.next().unwrap() as u8 {
        0 => None,
        1 => {
            let size = read_number(str, 4);
            let ptrs = read_number(str, 4);
            Some((size, ptrs))
        }
        _ => panic!("Invalid stack size flag"),
    };
    let len = read_number(str, 8);
    let mut params = Vec::with_capacity(len);
    for _ in 0..len {
        let flag = str.next().unwrap();
        let loc = read_number(str, 8);
        params.push(match flag as u8 {
            0 => MemoryLoc::Stack(loc),
            1 => MemoryLoc::Register(loc),
            _ => panic!("Invalid memory location flag"),
        });
    }
    FunSpec {
        name,
        loc,
        stack_size,
        params,
    }
}

pub fn push_str(source: &str, dest: &mut String) {
    dest.push_str(&b256str(source.len(), 8));
    for char in source.chars() {
        dest.push(char);
    }
}

pub fn push_chars(source: &Vec<char>, dest: &mut String) {
    dest.push_str(&b256str(source.len(), 8));
    for char in source.iter() {
        dest.push(*char);
    }
}

pub fn read_non_prim(str: &mut std::iter::Peekable<std::str::Chars<'_>>) -> NonPrimitiveType {
    let kind = read_number(str, 1);
    let len = read_number(str, 8);
    let name = read_str(str);
    let pointers = read_number(str, 8);
    let methods_len = read_number(str, 8);
    let mut methods = HashMap::with_capacity(methods_len);
    for _ in 0..methods_len {
        let trt = read_number(str, 8);
        let methods_len = read_number(str, 8);
        let mut mtds = Vec::with_capacity(methods_len);
        for _ in 0..methods_len {
            mtds.push(read_number(str, 8));
        }
        methods.insert(trt, mtds);
    }
    NonPrimitiveType {
        kind: match kind {
            0 => NonPrimitiveTypes::Array,
            1 => NonPrimitiveTypes::Struct,
            _ => panic!("Invalid non-primitive type"),
        },
        len,
        name,
        pointers,
        methods,
    }
}

fn read_str(str: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let len = read_number(str, 8);
    let mut res = String::with_capacity(len);
    for _ in 0..len {
        res.push(str.next().unwrap());
    }
    res
}

pub fn non_prim_into_string(non_prim: &NonPrimitiveType, str: &mut String) {
    let kind = match &non_prim.kind {
        NonPrimitiveTypes::Array => 0,
        NonPrimitiveTypes::Struct => 1,
    };
    str.push_str(&b256str(kind, 1));
    str.push_str(&b256str(non_prim.len, 8));
    push_str(&non_prim.name, str);
    str.push_str(&b256str(non_prim.pointers, 8));
    let len = non_prim.methods.len();
    str.push_str(&b256str(len, 8));
    for (trt, methods) in non_prim.methods.iter() {
        str.push_str(&b256str(*trt, 8));
        str.push_str(&b256str(methods.len(), 8));
        for method in methods.iter() {
            str.push_str(&b256str(*method, 8));
        }
    }
}

fn read_number(str: &mut std::iter::Peekable<std::str::Chars<'_>>, len: usize) -> usize {
    let mut number = 0;
    for _ in 0..len {
        number *= 256;
        number += str.next().unwrap() as usize;
    }

    number
}

pub fn byte_into_string(byte: Instructions, str: &mut String) {
    let append = match byte {
        Instructions::Debug(n) => s(0) + &b256str(n, 1),
        Instructions::Write(n1, n2) => s(1) + &b256str(n1, 4) + &b256str(n2, 1),
        Instructions::Read(n1, n2) => s(2) + &b256str(n1, 4) + &b256str(n2, 1),
        Instructions::WritePtr(n) => s(3) + &b256str(n, 1),
        Instructions::ReadPtr(n) => s(4) + &b256str(n, 1),
        Instructions::ReadConst(n1, n2) => s(5) + &b256str(n1, 4) + &b256str(n2, 1),
        Instructions::Ptr(n) => s(6) + &b256str(n, 4),
        Instructions::Index(n) => s(7) + &b256str(n, 4),
        Instructions::Allocate(n) => s(8) + &b256str(n, 1),
        Instructions::Reallocate(n) => s(9) + &b256str(n, 1),
        Instructions::Deallocate => s(10),
        Instructions::Goto(n) => s(11) + &b256str(n, 4),
        Instructions::GotoPtr => s(12),
        Instructions::Branch(n1, n2) => s(13) + &b256str(n1, 4) + &b256str(n2, 4),
        Instructions::Return => s(14),
        Instructions::Unfreeze => s(15),
        Instructions::ReserveStack(n1, n2) => s(16) + &b256str(n1, 4) + &b256str(n2, 4),
        Instructions::Swap(n1, n2) => s(17) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::Add(n1, n2, n3) => {
            s(18) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Sub(n1, n2, n3) => {
            s(19) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Mul(n1, n2, n3) => {
            s(20) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Div(n1, n2, n3) => {
            s(21) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Mod(n1, n2, n3) => {
            s(22) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Equ(n1, n2, n3) => {
            s(23) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Grt(n1, n2, n3) => {
            s(24) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Less(n1, n2, n3) => {
            s(25) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::And(n1, n2, n3) => {
            s(26) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Or(n1, n2, n3) => s(27) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1),
        Instructions::Not(n1, n2) => s(28) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::Cal(n1, n2) => s(29) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::End => s(30),
        Instructions::Cast(n1, n2) => s(31) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::Len(n) => s(32) + &b256str(n, 1),
        Instructions::Type(n1, n2) => s(33) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::Jump(n) => s(34) + &b256str(n, 4),
        Instructions::Freeze => s(35),
        Instructions::Back => s(36),
        Instructions::Move(n1, n2) => s(37) + &b256str(n1, 1) + &b256str(n2, 1),
        Instructions::Sweep => s(38),
        Instructions::SweepUnoptimized => s(39),
        Instructions::AllocateStatic(n) => s(40) + &b256str(n, 4),
        Instructions::IndexStatic(n) => s(41) + &b256str(n, 4),
        Instructions::FillRange(n1, n2) => s(42) + &b256str(n1, 1) + &b256str(n2, 4),
        Instructions::CopyRange(n1, n2, n3) => {
            s(43) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 4)
        }
        Instructions::Break(n) => s(44) + &b256str(n, 1),
        Instructions::DynMethod(n1, n2, n3) => {
            s(45) + &b256str(n1, 1) + &b256str(n2, 4) + &b256str(n3, 4)
        }
        Instructions::Panic => s(46),
        Instructions::Catch => s(47),
        Instructions::CatchId(n) => s(48) + &b256str(n, 4),
        Instructions::DeleteCatch(n) => s(49) + &b256str(n, 1),
        Instructions::NonPrimitiveType(n1, n2) => s(50) + &b256str(n1, 1) + &b256str(n2, 4),
        Instructions::StrNew => s(51),
        Instructions::IntoStr(n) => s(52) + &b256str(n, 1),
        Instructions::DynReserve(n) => s(53) + &b256str(n, 1),
        Instructions::DynArgument(n1, n2, n3) => {
            s(54) + &b256str(n1, 1) + &b256str(n2, 1) + &b256str(n3, 1)
        }
        Instructions::Neg(n) => s(55) + &b256str(n, 1),
        Instructions::WriteArg(pos /*0-15*/, reg) => {
            s(56) + &b256str(pos, 1) + &b256str(reg, 1)
        }
        Instructions::ReadArg(pos /*0-15*/, reg) => {
            s(57) + &b256str(pos, 1) + &b256str(reg, 1)
        }
        Instructions::OpenArgs => s(58),
        Instructions::CloseArgs => s(59),
        Instructions::NullCheck => s(60),
    };
    str.push_str(&append);
}

pub fn str_into_byte(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Instructions {
    let code = chars.next().unwrap() as u8;
    let byte = match code {
        0 => Instructions::Debug(read_number(chars, 1)),
        1 => Instructions::Write(read_number(chars, 4), read_number(chars, 1)),
        2 => Instructions::Read(read_number(chars, 4), read_number(chars, 1)),
        3 => Instructions::WritePtr(read_number(chars, 1)),
        4 => Instructions::ReadPtr(read_number(chars, 1)),
        5 => Instructions::ReadConst(read_number(chars, 4), read_number(chars, 1)),
        6 => Instructions::Ptr(read_number(chars, 4)),
        7 => Instructions::Index(read_number(chars, 4)),
        8 => Instructions::Allocate(read_number(chars, 1)),
        9 => Instructions::Reallocate(read_number(chars, 1)),
        10 => Instructions::Deallocate,
        11 => Instructions::Goto(read_number(chars, 4)),
        12 => Instructions::GotoPtr,
        13 => Instructions::Branch(read_number(chars, 4), read_number(chars, 4)),
        14 => Instructions::Return,
        15 => Instructions::Unfreeze,
        16 => Instructions::ReserveStack(read_number(chars, 4), read_number(chars, 4)),
        17 => Instructions::Swap(read_number(chars, 1), read_number(chars, 1)),
        18 => Instructions::Add(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        19 => Instructions::Sub(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        20 => Instructions::Mul(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        21 => Instructions::Div(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        22 => Instructions::Mod(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        23 => Instructions::Equ(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        24 => Instructions::Grt(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        25 => Instructions::Less(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        26 => Instructions::And(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        27 => Instructions::Or(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        28 => Instructions::Not(read_number(chars, 1), read_number(chars, 1)),
        29 => Instructions::Cal(read_number(chars, 1), read_number(chars, 1)),
        30 => Instructions::End,
        31 => Instructions::Cast(read_number(chars, 1), read_number(chars, 1)),
        32 => Instructions::Len(read_number(chars, 1)),
        33 => Instructions::Type(read_number(chars, 1), read_number(chars, 1)),
        34 => Instructions::Jump(read_number(chars, 4)),
        35 => Instructions::Freeze,
        36 => Instructions::Back,
        37 => Instructions::Move(read_number(chars, 1), read_number(chars, 1)),
        38 => Instructions::Sweep,
        39 => Instructions::SweepUnoptimized,
        40 => Instructions::AllocateStatic(read_number(chars, 4)),
        41 => Instructions::IndexStatic(read_number(chars, 4)),
        42 => Instructions::FillRange(read_number(chars, 1), read_number(chars, 4)),
        43 => Instructions::CopyRange(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 4),
        ),
        44 => Instructions::Break(read_number(chars, 1)),
        45 => Instructions::DynMethod(
            read_number(chars, 1),
            read_number(chars, 4),
            read_number(chars, 4),
        ),
        46 => Instructions::Panic,
        47 => Instructions::Catch,
        48 => Instructions::CatchId(read_number(chars, 4)),
        49 => Instructions::DeleteCatch(read_number(chars, 1)),
        50 => Instructions::NonPrimitiveType(read_number(chars, 1), read_number(chars, 4)),
        51 => Instructions::StrNew,
        52 => Instructions::IntoStr(read_number(chars, 1)),
        53 => Instructions::DynReserve(read_number(chars, 1)),
        54 => Instructions::DynArgument(
            read_number(chars, 1),
            read_number(chars, 1),
            read_number(chars, 1),
        ),
        55 => Instructions::Neg(read_number(chars, 1)),
        56 => Instructions::WriteArg(read_number(chars, 1), read_number(chars, 1)),
        57 => Instructions::ReadArg(read_number(chars, 1), read_number(chars, 1)),
        58 => Instructions::OpenArgs,
        59 => Instructions::CloseArgs,
        60 => Instructions::NullCheck,
        _ => panic!("Unknown instruction"),
    };
    byte
}

pub fn value_into_byte(value: Types, str: &mut String) {
    let res = match value {
        Types::Int(n) => {
            s(0) + &b256str(
                unsafe { std::mem::transmute::<i64, usize>(n) },
                8,
            )
        }
        Types::Float(n) => s(1) + &b256str(unsafe { std::mem::transmute::<f64, usize>(n) }, 8),
        Types::Uint(n) => s(2) + &b256str(n, 8),
        Types::Char(n) => s(3) + &b256str(n as usize, 1),
        Types::Bool(n) => s(4) + &b256str(n as usize, 1),
        Types::Pointer(n, t) => s(5) + &b256str(n, 8) + &ptr_type_into_str(&t),
        Types::Function(n) => s(6) + &b256str(n, 8),
        Types::Null => s(7),
        Types::Void => s(8),
        Types::NonPrimitive(n) => s(9) + &b256str(n, 8),
    };
    str.push_str(&res);
}

fn bytes_into_value(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Types {
    let byte = chars.next().unwrap();
    match byte as u8 {
        0 => Types::Int(unsafe { 
            let num = read_number(chars, 8);
            std::mem::transmute::<usize, i64>(num) 
        }),
        1 => Types::Float(unsafe { std::mem::transmute::<usize, f64>(read_number(chars, 8)) }),
        2 => Types::Uint(read_number(chars, 8)),
        3 => Types::Char(read_number(chars, 1) as u8 as char),
        4 => Types::Bool(read_number(chars, 1) != 0),
        5 => Types::Pointer(read_number(chars, 8), read_ptr_type(chars)),
        6 => Types::Function(read_number(chars, 8)),
        7 => Types::Null,
        8 => Types::Void,
        9 => Types::NonPrimitive(read_number(chars, 8)),
        _ => panic!("Unknown type"),
    }
}

fn ptr_type_into_str(t: &PointerTypes) -> String {
    match &t {
        PointerTypes::String => s(0),
        PointerTypes::Object => s(1),
        PointerTypes::Stack => s(2),
        PointerTypes::Char(n) => s(3) + &b256str(*n, 8),
        PointerTypes::Heap(n) => s(4) + &b256str(*n, 8),
        PointerTypes::UserData => s(5),
    }
}

fn read_ptr_type(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> PointerTypes {
    let byte = chars.next().unwrap();
    match byte as u8 {
        0 => PointerTypes::String,
        1 => PointerTypes::Object,
        2 => PointerTypes::Stack,
        3 => PointerTypes::Char(read_number(chars, 8)),
        4 => PointerTypes::Heap(read_number(chars, 8)),
        5 => PointerTypes::UserData,
        _ => panic!("Unknown pointer type the program will not continue"),
    }
}

pub fn into_base256(mut n: usize, fill_size: usize) -> Vec<u8> {
    if n == 0 {
        return vec![0; fill_size];
    }
    if n > 256usize.pow(fill_size as u32) - 1 {
        println!(
            "Important! number {} is too large to fit in {} bytes ({} max number) program will continue with corrupted data",
            n, fill_size, 256usize.pow(fill_size as u32) - 1
        );
    }

    let mut vec = Vec::new();
    for _ in 0..fill_size {
        vec.push((n % 256) as u8);
        n >>= 8;
    }
    vec.reverse();
    vec
}

pub fn base256_to_string(vec: Vec<u8>) -> String {
    let mut string = String::with_capacity(vec.len());
    for byte in vec {
        string.push(byte as char);
    }
    string
}

pub fn b256str(n: usize, fill_size: usize) -> String {
    base256_to_string(into_base256(n, fill_size))
}

pub fn b(instr: u8) -> char {
    instr as char
}

pub fn s(instr: u8) -> String {
    b(instr).to_string()
}
