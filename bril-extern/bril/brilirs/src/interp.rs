use crate::basic_block::{BBFunction, BBProgram, BasicBlock};
use crate::error::{InterpError, PositionalInterpError};
use bril_rs::Instruction;
use bril2json::escape_control_chars;

use fxhash::FxHashMap;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::cmp::max;
use std::collections::HashMap;
use std::fmt;

// The Environment is the data structure used to represent the stack of the program.
// The values of all variables are store here. Each variable is represented as a number so
// each value can be store at the index of that number.
// Each function call gets allocated a "frame" which is just the offset that each variable
// should be index from for the duration of that call.
//  Call "main" pointer(frame size 3)
//  |
//  |        Call "foo" pointer(frame size 2)
//  |        |
// [a, b, c, a, b]
struct Environment {
  // Pointer into env for the start of the current frame
  current_pointer: usize,
  // Size of the current frame
  current_frame_size: usize,
  // A list of all stack pointers for valid frames on the stack
  stack_pointers: Vec<(usize, usize)>,
  // env is used like a stack. Assume it only grows
  env: Vec<Value>,
}

impl Environment {
  pub fn new(size: usize) -> Self {
    Self {
      current_pointer: 0,
      current_frame_size: size,
      stack_pointers: Vec::new(),
      // Allocate a larger stack size so the interpreter needs to allocate less often
      env: vec![Value::default(); max(size, 50)],
    }
  }

  pub fn get(&self, ident: usize) -> &Value {
    // A bril program is well formed when, dynamically, every variable is defined before its use.
    // If this is violated, this will return Value::Uninitialized and the whole interpreter will come crashing down.
    self.env.get(self.current_pointer + ident).unwrap()
  }

  // Used for getting arguments that should be passed to the current frame from the previous one
  pub fn get_from_last_frame(&self, ident: usize) -> &Value {
    let past_pointer = self.stack_pointers.last().unwrap().0;
    self.env.get(past_pointer + ident).unwrap()
  }

  pub fn set(&mut self, ident: usize, val: Value) {
    self.env[self.current_pointer + ident] = val;
  }
  // Push a new frame onto the stack
  pub fn push_frame(&mut self, size: usize) {
    self
      .stack_pointers
      .push((self.current_pointer, self.current_frame_size));
    self.current_pointer += self.current_frame_size;
    self.current_frame_size = size;

    // Check that the stack is large enough
    if self.current_pointer + self.current_frame_size > self.env.len() {
      // We need to allocate more stack
      self.env.resize(
        max(
          self.env.len() * 4,
          self.current_pointer + self.current_frame_size,
        ),
        Value::default(),
      );
    }
  }

  // Remove a frame from the stack
  pub fn pop_frame(&mut self) {
    (self.current_pointer, self.current_frame_size) = self.stack_pointers.pop().unwrap();
  }
}

// todo: This is basically a copy of the heap implement in brili and we could probably do something smarter. This currently isn't that worth it to optimize because most benchmarks do not use the memory extension nor do they run for very long. You (the reader in the future) may be working with bril programs that you would like to speed up that extensively use the bril memory extension. In that case, it would be worth seeing how to implement Heap without a map based memory. Maybe try to re-implement malloc for a large Vec<Value>?
struct Heap {
  memory: FxHashMap<usize, Vec<Value>>,
  base_num_counter: usize,
}

impl Default for Heap {
  fn default() -> Self {
    Self {
      memory: FxHashMap::with_capacity_and_hasher(20, fxhash::FxBuildHasher::default()),
      base_num_counter: 0,
    }
  }
}

impl Heap {
  fn is_empty(&self) -> bool {
    self.memory.is_empty()
  }

  fn alloc(&mut self, amount: i64) -> Result<Value, InterpError> {
    let amount: usize = amount
      .try_into()
      .map_err(|_| InterpError::CannotAllocSize(amount))?;
    let base = self.base_num_counter;
    self.base_num_counter += 1;
    self.memory.insert(base, vec![Value::default(); amount]);
    Ok(Value::Pointer(Pointer { base, offset: 0 }))
  }

  fn free(&mut self, key: &Pointer) -> Result<(), InterpError> {
    if self.memory.remove(&key.base).is_some() && key.offset == 0 {
      Ok(())
    } else {
      Err(InterpError::IllegalFree(key.base, key.offset))
    }
  }

  fn write(&mut self, key: &Pointer, val: Value) -> Result<(), InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    match self.memory.get_mut(&key.base) {
      Some(vec) if vec.len() > offset => {
        vec[offset] = val;
        Ok(())
      }
      Some(_) | None => Err(InterpError::InvalidMemoryAccess(key.base, key.offset)),
    }
  }

  fn read(&self, key: &Pointer) -> Result<&Value, InterpError> {
    // Will check that key.offset is >=0
    let offset: usize = key
      .offset
      .try_into()
      .map_err(|_| InterpError::InvalidMemoryAccess(key.base, key.offset))?;
    self
      .memory
      .get(&key.base)
      .and_then(|vec| vec.get(offset))
      .ok_or(InterpError::InvalidMemoryAccess(key.base, key.offset))
      .and_then(|val| match val {
        Value::Uninitialized => Err(InterpError::UsingUninitializedMemory),
        _ => Ok(val),
      })
  }
}

// A getter function for when you know what constructor of the Value enum you have and
// you just want the underlying value(like a f64).
// Or can just be used to get a owned version of the Value
fn get_arg<'a, T: From<&'a Value>>(vars: &'a Environment, index: usize, args: &[usize]) -> T {
  T::from(vars.get(args[index]))
}

#[derive(Debug, Default, Clone, Copy)]
enum Value {
  Int(i64),
  Bool(bool),
  Float(f64),
  Char(char),
  Pointer(Pointer),
  #[default]
  Uninitialized,
}

#[derive(Debug, Clone, PartialEq, Copy)]
struct Pointer {
  base: usize,
  offset: i64,
}

impl Pointer {
  const fn add(&self, offset: i64) -> Self {
    Self {
      base: self.base,
      offset: self.offset + offset,
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Int(i) => write!(f, "{i}"),
      Self::Bool(b) => write!(f, "{b}"),
      Self::Float(v) if v.is_infinite() && v.is_sign_positive() => write!(f, "Infinity"),
      Self::Float(v) if v.is_infinite() && v.is_sign_negative() => write!(f, "-Infinity"),
      Self::Float(v) if v != &0.0 && v.abs().log10() >= 10.0 => {
        f.write_str(format!("{v:.17e}").replace('e', "e+").as_str())
      }
      Self::Float(v) if v != &0.0 && v.abs().log10() <= -10.0 => write!(f, "{v:.17e}"),
      Self::Float(v) => write!(f, "{v:.17}"),
      Self::Char(c) => write!(f, "{c}"),
      Self::Pointer(p) => write!(f, "{p:?}"),
      Self::Uninitialized => unreachable!(),
    }
  }
}

fn optimized_val_output<T: std::io::Write>(out: &mut T, val: &Value) -> Result<(), std::io::Error> {
  match val {
    Value::Int(i) => out.write_all(itoa::Buffer::new().format(*i).as_bytes()),
    Value::Bool(b) => out.write_all(if *b { b"true" } else { b"false" }),
    Value::Float(f) if f.is_infinite() && f.is_sign_positive() => out.write_all(b"Infinity"),
    Value::Float(f) if f.is_infinite() && f.is_sign_negative() => out.write_all(b"-Infinity"),
    Value::Float(f) if f.is_nan() => out.write_all(b"NaN"),
    Value::Float(f) if f != &0.0 && f.abs().log10() >= 10.0 => {
      out.write_all(format!("{f:.17e}").replace('e', "e+").as_bytes())
    }
    Value::Float(f) if f != &0.0 && f.abs().log10() <= -10.0 => {
      out.write_all(format!("{f:.17e}").as_bytes())
    }
    Value::Float(f) => out.write_all(format!("{f:.17}").as_bytes()),
    Value::Char(c) => {
      let buf = &mut [0_u8; 2];
      out.write_all(c.encode_utf8(buf).as_bytes())
    }
    Value::Pointer(p) => out.write_all(format!("{p:?}").as_bytes()),
    Value::Uninitialized => unreachable!(),
  }
}

impl From<&bril_rs::Literal> for Value {
  fn from(l: &bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(*i),
      bril_rs::Literal::Bool(b) => Self::Bool(*b),
      bril_rs::Literal::Float(f) => Self::Float(*f),
      bril_rs::Literal::Char(c) => Self::Char(*c),
    }
  }
}

impl From<bril_rs::Literal> for Value {
  fn from(l: bril_rs::Literal) -> Self {
    match l {
      bril_rs::Literal::Int(i) => Self::Int(i),
      bril_rs::Literal::Bool(b) => Self::Bool(b),
      bril_rs::Literal::Float(f) => Self::Float(f),
      bril_rs::Literal::Char(c) => Self::Char(c),
    }
  }
}

impl From<&Value> for i64 {
  fn from(value: &Value) -> Self {
    if let Value::Int(i) = value {
      *i
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for bool {
  fn from(value: &Value) -> Self {
    if let Value::Bool(b) = value {
      *b
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for f64 {
  fn from(value: &Value) -> Self {
    if let Value::Float(f) = value {
      *f
    } else {
      unreachable!()
    }
  }
}

impl From<&Value> for char {
  fn from(value: &Value) -> Self {
    if let Value::Char(c) = value {
      *c
    } else {
      unreachable!()
    }
  }
}

impl<'a> From<&'a Value> for &'a Pointer {
  fn from(value: &'a Value) -> Self {
    if let Value::Pointer(p) = value {
      p
    } else {
      unreachable!()
    }
  }
}

impl From<&Self> for Value {
  fn from(value: &Self) -> Self {
    *value
  }
}

// Sets up the Environment for the next function call with the supplied arguments
fn make_func_args(callee_func: &BBFunction, args: &[usize], vars: &mut Environment) {
  vars.push_frame(callee_func.num_of_vars);

  args
    .iter()
    .zip(callee_func.args_as_nums.iter())
    .for_each(|(arg_name, expected_arg)| {
      let arg = vars.get_from_last_frame(*arg_name);
      vars.set(*expected_arg, *arg);
    });
}

fn execute_value_op<T: std::io::Write>(
  state: &mut State<T>,
  op: bril_rs::ValueOps,
  dest: usize,
  args: &[usize],
  funcs: &[usize],
  shadow_env: &HashMap<usize, Value>,
) -> Result<(), InterpError> {
  use bril_rs::ValueOps::{
    Add, Alloc, And, Bits2Float, Call, Ceq, Cge, Cgt, Char2int, Cle, Clt, Div, Eq, Fadd, Fdiv, Feq,
    Fge, Fgt, Fle, Float2Bits, Flt, Fmul, Fsub, Ge, Get, Gt, Id, Int2char, Le, Load, Lt, Mul, Not,
    Or, PtrAdd, Sub, Undef,
  };
  match op {
    Add => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_add(arg1)));
    }
    Mul => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_mul(arg1)));
    }
    Sub => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Int(arg0.wrapping_sub(arg1)));
    }
    Div => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      if arg1 == 0 {
        return Err(InterpError::DivisionByZero);
      }
      state.env.set(dest, Value::Int(arg0.wrapping_div(arg1)));
    }
    Eq => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Lt => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Gt => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Le => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Ge => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Not => {
      let arg0 = get_arg::<bool>(&state.env, 0, args);
      state.env.set(dest, Value::Bool(!arg0));
    }
    And => {
      let arg0 = get_arg::<bool>(&state.env, 0, args);
      let arg1 = get_arg::<bool>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 && arg1));
    }
    Or => {
      let arg0 = get_arg::<bool>(&state.env, 0, args);
      let arg1 = get_arg::<bool>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 || arg1));
    }
    Id => {
      let src = get_arg::<Value>(&state.env, 0, args);
      state.env.set(dest, src);
    }
    Fadd => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 + arg1));
    }
    Fmul => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 * arg1));
    }
    Fsub => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 - arg1));
    }
    Fdiv => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Float(arg0 / arg1));
    }
    Feq => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Flt => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Fgt => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Fle => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Fge => {
      let arg0 = get_arg::<f64>(&state.env, 0, args);
      let arg1 = get_arg::<f64>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Ceq => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      let arg1 = get_arg::<char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 == arg1));
    }
    Clt => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      let arg1 = get_arg::<char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 < arg1));
    }
    Cgt => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      let arg1 = get_arg::<char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 > arg1));
    }
    Cle => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      let arg1 = get_arg::<char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 <= arg1));
    }
    Cge => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      let arg1 = get_arg::<char>(&state.env, 1, args);
      state.env.set(dest, Value::Bool(arg0 >= arg1));
    }
    Char2int => {
      let arg0 = get_arg::<char>(&state.env, 0, args);
      state.env.set(dest, Value::Int(u32::from(arg0).into()));
    }
    Int2char => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);

      let arg0_char = u32::try_from(arg0)
        .ok()
        .and_then(char::from_u32)
        .ok_or(InterpError::ToCharError(arg0))?;

      state.env.set(dest, Value::Char(arg0_char));
    }
    Call => {
      let callee_func = state.prog.get(funcs[0]).unwrap();

      make_func_args(callee_func, args, &mut state.env);

      let result = execute(state, callee_func)?.unwrap();

      state.env.pop_frame();

      state.env.set(dest, result);
    }
    Get => match shadow_env.get(&dest) {
      Some(v) => state.env.set(dest, *v),
      None => return Err(InterpError::GetWithoutSet),
    },
    Undef => {
      state.env.set(dest, Value::Uninitialized);
    }
    Alloc => {
      let arg0 = get_arg::<i64>(&state.env, 0, args);
      let res = state.heap.alloc(arg0)?;
      state.env.set(dest, res);
    }
    Load => {
      let arg0 = get_arg::<&Pointer>(&state.env, 0, args);
      let res = state.heap.read(arg0)?;
      state.env.set(dest, *res);
    }
    PtrAdd => {
      let arg0 = get_arg::<&Pointer>(&state.env, 0, args);
      let arg1 = get_arg::<i64>(&state.env, 1, args);
      let res = Value::Pointer(arg0.add(arg1));
      state.env.set(dest, res);
    }
    Float2Bits => {
      let float = get_arg::<f64>(&state.env, 0, args);
      // https://users.rust-lang.org/t/i64-u64-mapping-revisited/109315
      // the to and from native endian stuff is a nop
      // if link dies try web archive
      let int = i64::from_ne_bytes(float.to_ne_bytes());
      state.env.set(dest, Value::Int(int));
    }
    Bits2Float => {
      let int = get_arg::<i64>(&state.env, 0, args);
      // see comment for Float2Bits
      let float = f64::from_ne_bytes(int.to_ne_bytes());
      state.env.set(dest, Value::Float(float));
    }
  }
  Ok(())
}

fn execute_effect_op<T: std::io::Write>(
  state: &mut State<T>,
  op: bril_rs::EffectOps,
  args: &[usize],
  funcs: &[usize],
  curr_block: &BasicBlock,
  // There are two output variables where values are stored to effect the loop execution.
  next_block_idx: &mut Option<usize>,
  result: &mut Option<Value>,
  shadow_env: &mut HashMap<usize, Value>,
) -> Result<(), InterpError> {
  use bril_rs::EffectOps::{
    Branch, Call, Commit, Free, Guard, Jump, Nop, Print, Return, Set, Speculate, Store,
  };
  match op {
    Jump => {
      *next_block_idx = Some(curr_block.exit[0]);
    }
    Branch => {
      let bool_arg0 = get_arg::<bool>(&state.env, 0, args);
      let exit_idx = usize::from(!bool_arg0);
      *next_block_idx = Some(curr_block.exit[exit_idx]);
    }
    Return => {
      if !args.is_empty() {
        *result = Some(get_arg::<Value>(&state.env, 0, args));
      }
    }
    Print => {
      // In the typical case, users only print out one value at a time
      // So we can usually avoid extra allocations by providing that string directly
      if args.len() == 1 {
        optimized_val_output(&mut state.out, state.env.get(*args.first().unwrap()))?;
        // Add new line
        state.out.write_all(b"\n")?;
      } else {
        writeln!(
          state.out,
          "{}",
          args
            .iter()
            .map(|a| state.env.get(*a).to_string())
            .collect::<Vec<String>>()
            .join(" ")
        )?;
      }
    }
    Nop => {}
    Call => {
      let callee_func = state.prog.get(funcs[0]).unwrap();

      make_func_args(callee_func, args, &mut state.env);

      execute(state, callee_func)?;
      state.env.pop_frame();
    }
    Store => {
      let arg0 = get_arg::<&Pointer>(&state.env, 0, args);
      let arg1 = get_arg::<Value>(&state.env, 1, args);
      state.heap.write(arg0, arg1)?;
    }
    Free => {
      let arg0 = get_arg::<&Pointer>(&state.env, 0, args);
      state.heap.free(arg0)?;
    }
    Set => {
      let arg = get_arg::<Value>(&state.env, 1, args);
      shadow_env.insert(args[0], arg);
    }
    Speculate | Commit | Guard => unimplemented!(),
  }
  Ok(())
}

fn execute<'a, T: std::io::Write>(
  state: &mut State<'a, T>,
  func: &'a BBFunction,
) -> Result<Option<Value>, PositionalInterpError> {
  let mut shadow_env = HashMap::new();
  let mut curr_block_idx = 0;
  // A possible return value
  let mut result = None;

  loop {
    let curr_block = &func.blocks[curr_block_idx];
    let curr_instrs = &curr_block.instrs;
    let curr_numified_instrs = &curr_block.numified_instrs;
    // WARNING!!! We can add the # of instructions at once because you can only jump to a new block at the end. This may need to be changed if speculation is implemented
    state.instruction_count += curr_instrs.len();

    // A place to store the next block that will be jumped to if specified by an instruction
    let mut next_block_idx = None;

    for (code, numified_code) in curr_instrs.iter().zip(curr_numified_instrs.iter()) {
      match code {
        Instruction::Constant {
          op: bril_rs::ConstOps::Const,
          dest: _,
          const_type,
          value,
          pos: _,
        } => {
          // Integer literals can be promoted to Floating point
          if const_type == &bril_rs::Type::Float {
            match value {
              #[expect(
                clippy::cast_precision_loss,
                reason = "So yes, as clippy points out, you technically lose precision here on the `*i as f64` cast. On the other hand, you already give up precision when you start using floats and I haven't been able to find a case where you are giving up precision in the cast that you don't already lose by using floating points. So it's probably fine unless proven otherwise."
              )]
              bril_rs::Literal::Int(i) => state
                .env
                .set(numified_code.dest.unwrap(), Value::Float(*i as f64)),
              bril_rs::Literal::Float(f) => {
                state.env.set(numified_code.dest.unwrap(), Value::Float(*f));
              }
              bril_rs::Literal::Char(_) | bril_rs::Literal::Bool(_) => unreachable!(),
            }
          } else {
            state
              .env
              .set(numified_code.dest.unwrap(), Value::from(value));
          };
        }
        Instruction::Value {
          op,
          dest: _,
          op_type: _,
          args: _,
          labels: _,
          funcs: _,
          pos,
        } => {
          execute_value_op(
            state,
            *op,
            numified_code.dest.unwrap(),
            &numified_code.args,
            &numified_code.funcs,
            &shadow_env,
          )
          .map_err(|e| e.add_pos(pos.clone()))?;
        }
        Instruction::Effect {
          op,
          args: _,
          labels: _,
          funcs: _,
          pos,
        } => {
          execute_effect_op(
            state,
            *op,
            &numified_code.args,
            &numified_code.funcs,
            curr_block,
            &mut next_block_idx,
            &mut result,
            &mut shadow_env,
          )
          .map_err(|e| e.add_pos(pos.clone()))?;
        }
      }
    }

    // Are we jumping to a new block or are we done?
    if let Some(idx) = next_block_idx {
      curr_block_idx = idx;
    } else if curr_block.exit.len() == 1 {
      curr_block_idx = curr_block.exit[0];
    } else {
      return Ok(result);
    }
  }
}

fn parse_args(
  mut env: Environment,
  args: &[bril_rs::Argument],
  args_as_nums: &[usize],
  inputs: &[String],
) -> Result<Environment, InterpError> {
  if args.is_empty() && inputs.is_empty() {
    Ok(env)
  } else if inputs.len() != args.len() {
    Err(InterpError::BadNumFuncArgs(args.len(), inputs.len()))
  } else {
    args
      .iter()
      .zip(args_as_nums.iter())
      .enumerate()
      .try_for_each(|(index, (arg, arg_as_num))| match arg.arg_type {
        bril_rs::Type::Bool => {
          match inputs.get(index).unwrap().parse::<bool>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Bool,
                (*inputs.get(index).unwrap()).to_string(),
              ));
            }
            Ok(b) => env.set(*arg_as_num, Value::Bool(b)),
          };
          Ok(())
        }
        bril_rs::Type::Int => {
          match inputs.get(index).unwrap().parse::<i64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Int,
                (*inputs.get(index).unwrap()).to_string(),
              ));
            }
            Ok(i) => env.set(*arg_as_num, Value::Int(i)),
          };
          Ok(())
        }
        bril_rs::Type::Float => {
          match inputs.get(index).unwrap().parse::<f64>() {
            Err(_) => {
              return Err(InterpError::BadFuncArgType(
                bril_rs::Type::Float,
                (*inputs.get(index).unwrap()).to_string(),
              ));
            }
            Ok(f) => env.set(*arg_as_num, Value::Float(f)),
          };
          Ok(())
        }
        bril_rs::Type::Char => escape_control_chars(inputs.get(index).unwrap().as_ref())
          .map_or_else(
            || Err(InterpError::NotOneChar),
            |c| {
              env.set(*arg_as_num, Value::Char(c));
              Ok(())
            },
          ),
        bril_rs::Type::Pointer(..) | bril_rs::Type::Any => unreachable!(),
      })?;
    Ok(env)
  }
}

// State captures the parts of the interpreter that are used across function boundaries
struct State<'a, T: std::io::Write> {
  prog: &'a BBProgram,
  env: Environment,
  heap: Heap,
  out: T,
  instruction_count: usize,
}

impl<'a, T: std::io::Write> State<'a, T> {
  const fn new(prog: &'a BBProgram, env: Environment, heap: Heap, out: T) -> Self {
    Self {
      prog,
      env,
      heap,
      out,
      instruction_count: 0,
    }
  }
}

/// The entrance point to the interpreter.
///
/// It runs over a ```prog```:[`BBProgram`] starting at the "main" function with ```input_args``` as input. Print statements output to ```out``` which implements [`std::io::Write`]. You also need to include whether you want the interpreter to count the number of instructions run with ```profiling```. This information is outputted to [`std::io::stderr`]
/// # Panics
/// This should not panic with normal use except if there is a bug or if you are using an unimplemented feature
/// # Errors
/// Will error on malformed `BBProgram`, like if the original Bril program was not well-formed
pub fn execute_main<T: std::io::Write, U: std::io::Write>(
  prog: &BBProgram,
  out: T,
  input_args: &[String],
  profiling: bool,
  mut profiling_out: U,
) -> Result<(), PositionalInterpError> {
  let main_func = prog
    .index_of_main
    .map(|i| prog.get(i).unwrap())
    .ok_or(InterpError::NoMainFunction)?;

  let mut env = Environment::new(main_func.num_of_vars);
  let heap = Heap::default();

  env = parse_args(env, &main_func.args, &main_func.args_as_nums, input_args)
    .map_err(|e| e.add_pos(main_func.pos.clone()))?;

  let mut state = State::new(prog, env, heap, out);

  execute(&mut state, main_func)?;

  if !state.heap.is_empty() {
    return Err(InterpError::MemLeak).map_err(|e| e.add_pos(main_func.pos.clone()));
  }

  state.out.flush().map_err(InterpError::IoError)?;

  if profiling {
    writeln!(profiling_out, "total_dyn_inst: {}", state.instruction_count)
      // We call flush here in case `profiling_out` is a https://doc.rust-lang.org/std/io/struct.BufWriter.html
      // Otherwise we would expect this flush to be a nop.
      .and_then(|()| profiling_out.flush())
      .map_err(InterpError::IoError)?;
  }

  Ok(())
}
