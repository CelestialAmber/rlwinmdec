use anyhow::{bail, Result};

#[derive(Debug, Copy, Clone, PartialEq)]
enum InstructionType {
	Rlwinm,
	Extlwi,
	Extrwi,
	Rotlwi,
	Rotrwi,
	Slwi,
	Srwi,
	Clrlwi,
	Clrrwi,
	Clrlslwi,
	Rlwimi,
	Inslwi,
	Insrwi,
	Rlwnm,
	Rotlw
}

impl InstructionType {
	fn from_usize(val: usize) -> Result<InstructionType> {
		let result = match val {
			0 => InstructionType::Rlwinm,
			1 => InstructionType::Extlwi,
			2 => InstructionType::Extrwi,
			3 => InstructionType::Rotlwi,
			4 => InstructionType::Rotrwi,
			5 => InstructionType::Slwi,
			6 => InstructionType::Srwi,
			7 => InstructionType::Clrlwi,
			8 => InstructionType::Clrrwi,
			9 => InstructionType::Clrlslwi,
			10 => InstructionType::Rlwimi,
			11 => InstructionType::Inslwi,
			12 => InstructionType::Insrwi,
			13 => InstructionType::Rlwnm,
			14 => InstructionType::Rotlw,
			_ => {
				bail!("Invalid value")
			}
		};
		
		Ok(result)
	}
}

#[derive(Debug, Clone)]
struct InstructionData<'a> {
	name: &'a str,
	args: i32,
}

const INSTRUCTIONS : [InstructionData; 15] = [
	InstructionData{name: "rlwinm", args: 5},
	InstructionData{name: "extlwi", args: 4},
	InstructionData{name: "extrwi", args: 4},
	InstructionData{name: "rotlwi", args: 3},
	InstructionData{name: "rotrwi", args: 3},
	InstructionData{name: "slwi", args: 3},
	InstructionData{name: "srwi", args: 3},
	InstructionData{name: "clrlwi", args: 3},
	InstructionData{name: "clrrwi", args: 3},
	InstructionData{name: "clrlslwi", args: 4},
	InstructionData{name: "rlwimi", args: 5},
	InstructionData{name: "inslwi", args: 4},
	InstructionData{name: "insrwi", args: 4},
	InstructionData{name: "rlwnm", args: 5},
	InstructionData{name: "rotlw", args: 3}
];


pub fn decode(instruction : &str) -> Option<String> {
	let split = instruction.split(',');
	let mut instruction_type : InstructionType = InstructionType::Rlwinm;
	let mut instruction_name = "";
	let mut parts : Vec<String> = vec![];
	
	for part in split {
		parts.push(part.to_string());
	}
	
	let length = parts.len();
	let mut result: String = String::from("");
	let mut valid_instruction = false;

	for (i, instr) in INSTRUCTIONS.iter().enumerate() {
		if parts[0].contains(instr.name) && length == instr.args as usize {
			instruction_type = match InstructionType::from_usize(i) {
				Ok(result) => result,
				Err(_e) => { return None; }
			};
			instruction_name = instr.name;
			valid_instruction = true;
			break;
		}
	}

	if !valid_instruction {
		return None;
	}

	let mut sets_flags = false;

	if parts[0].contains('.') {
		sets_flags = true;
	}

	//Remove the instruction name from the first parameter string (also the dot for the version with the special dot after the name)
	parts[0] = parts[0].replace(instruction_name, "").replace('.', "");

	//Remove all spaces from each part of the instruction
	for part in parts.iter_mut() {
		*part = part.replace(' ', "");
	}

	let r_dest: &str = parts[0].as_str();
	let r_source: &str = parts[1].as_str();

	if !check_if_valid_reg_string(r_dest) || !check_if_valid_reg_string(r_source) {
		return None;
	}

	let mut shift_amount: u32 = 0;
	let mut bitmask_start: u32 = 0;
	let mut bitmask_end: u32 = 0;
	let mut rshift_amount = ""; //used for rlwnm/rotlw

		if instruction_type == InstructionType::Rlwinm || instruction_type == InstructionType::Rlwimi {
			for (i, part) in parts.iter().enumerate().take(5).skip(2) {
				let num_string = part.as_str();
				if let Ok(val) = num_string.parse::<u32>() {
					if i == 2 { shift_amount = val; }
					else if i == 3 { bitmask_start = val; }
					else if i == 4 { bitmask_end = val; }
				}else{
					return None;
				}
			}
		}else if instruction_type == InstructionType::Rlwnm {
			rshift_amount = parts[2].as_str();
			if !check_if_valid_reg_string(rshift_amount) {
				return None;
			}

			bitmask_start = match parts[3].parse::<u32>() {
				Ok(num) => num,
				Err(_e) => { return None; }
			};
			bitmask_end = match parts[4].parse::<u32>() {
				Ok(num) => num,
				Err(_e) => { return None; }
			};
		} else if instruction_type == InstructionType::Rotlwi || instruction_type == InstructionType::Rotrwi || instruction_type == InstructionType::Slwi || instruction_type == InstructionType::Srwi || instruction_type == InstructionType::Clrlwi || instruction_type == InstructionType::Clrrwi {
		   //rlwinm/rlwimi mnemonics w/ 3 arguments
			let num_string = &parts[2];
			let val = match num_string.parse::<u32>() {
				Ok(num) => num,
				Err(_e) => { return None; }
			};

			if instruction_type == InstructionType::Rotlwi{
				bitmask_start = 0;
				bitmask_end = 31;
				shift_amount = val;
			}else if instruction_type == InstructionType::Rotrwi {
				bitmask_start = 0;
				bitmask_end = 31;
				shift_amount = 32 - val;
			}else if instruction_type == InstructionType::Slwi {
				bitmask_start = 0;
				bitmask_end = 31 - val;
				shift_amount = val;
			}else if instruction_type == InstructionType::Srwi {
				bitmask_start = val;
				bitmask_end = 31;
				shift_amount = 32 - val;
			}else if instruction_type == InstructionType::Clrlwi {
				bitmask_start = val;
				bitmask_end = 31;
				shift_amount = 0;
			}else if instruction_type == InstructionType::Clrrwi {
				bitmask_start = 0;
				bitmask_end = 31 - val;
				shift_amount = 0;
			}
		}else if instruction_type == InstructionType::Extlwi || instruction_type == InstructionType::Extrwi || instruction_type == InstructionType::Clrlslwi || instruction_type == InstructionType::Inslwi || instruction_type == InstructionType::Insrwi {
			//rlwinm/rlwimi mnmemonics w/ 4 arguments
			let num_string1 = &parts[2];
			let num_string2 = &parts[3];
			let val1: u32 = match num_string1.parse::<u32>() {
				Ok(num) => num,
				Err(_e) => { return None; }
			};
			let val2: u32 = match num_string2.parse::<u32>() {
				Ok(num) => num,
				Err(_e) => { return None; }
			};

			if instruction_type == InstructionType::Extlwi { //rlwinm mnemonics
				bitmask_start = 0;
				bitmask_end = val1 - 1;
				shift_amount = val2;
			}else if instruction_type == InstructionType::Extrwi {
				bitmask_start = 32 - val1;
				bitmask_end = 31;
				shift_amount = val2 + val1;
			}else if instruction_type == InstructionType::Clrlslwi {
				bitmask_start = val1 - val2;
				bitmask_end = 31 - val2;
				shift_amount = val2;
			}else if instruction_type == InstructionType::Inslwi { //rlwimi mnemonics
				bitmask_start = val2;
				bitmask_end = val2 + val1 - 1;
				shift_amount = 32 - val2;
			}else if instruction_type == InstructionType::Insrwi {
				bitmask_start = val2;
				bitmask_end = (val2 + val1) - 1;
				shift_amount = 32 - (val2 + val1);
			}

		}else if instruction_type == InstructionType::Rotlw {
			//rotlw (rlwmn mnemonic)
			rshift_amount = parts[2].as_str();
			if !check_if_valid_reg_string(rshift_amount) {
				return None;
			}

			bitmask_start = 0;
			bitmask_end = 31;
		}

		if bitmask_start > 31 || bitmask_end > 31 {
			return None;
		}

	let bitmask: u32 = generate_bitmask(bitmask_start, bitmask_end);

	if instruction_type == InstructionType::Rlwinm || instruction_type == InstructionType::Rotlwi || instruction_type == InstructionType::Rotrwi || instruction_type == InstructionType::Slwi || instruction_type == InstructionType::Srwi || instruction_type == InstructionType::Clrlwi || instruction_type == InstructionType::Clrrwi || instruction_type == InstructionType::Extlwi || instruction_type == InstructionType::Extrwi || instruction_type == InstructionType::Clrlslwi {
	   //Rlwinm
		//If the destination and source registers are the same, and the shift amount is 0, then add &= (only anding with a given bitmask)
		if r_dest == r_source && shift_amount == 0 {
			result += format!("{} &= {:#X};\n", r_dest, bitmask).as_str();
			result += "Could also be:\n";
			result += format!("{} &= ~{:#X};\n", r_dest, !bitmask).as_str();
		} else if shift_amount == 0 {
			result += format!("{} = {} & {:#X};\n", r_dest, r_source, bitmask).as_str();
			result += "Could also be:\n";
			result += format!("{} = {} & ~{:#X};\n", r_dest, r_source, !bitmask).as_str();
		} else {
		  /* mwcc sometimes does an optimization where n*2^m will become rlwinm, where the zero bits are ensured to be 0
		  through anding with a bitmask (for example, n*4 becomes n<<2 & ~0x3, clearing the lower bits */
		  if bitmask == (!((1 << shift_amount) - 1)) {
			  result += format!("{} = {} << {};\n", r_dest, r_source, shift_amount).as_str();
			  result += "Could also be:\n";
			  result += format!("{} = {}*{};\n", r_dest, r_source, (1 << shift_amount)).as_str();
		  }else if bitmask == !(((1 << (32-shift_amount)) - 1) << shift_amount) {
			//for division, the same happens, except the top bits are cleared (bits are effectively right shifted through rlwinm)
			result += format!("{} = {} >> {};\n", r_dest, r_source, (32 - shift_amount)).as_str();
			result += "Could also be:\n";
			  result += format!("{} = {}/{};\n", r_dest, r_source, (1 << (32 - shift_amount))).as_str();
		  }else{
			result += format!("{} = ({} << {}) & {:#X};\n", r_dest, r_source, shift_amount, bitmask).as_str();
			result += "Could also be:\n";
			result += format!("{} = ({} << {}) & ~{:#X};\n", r_dest, r_source, shift_amount, !bitmask).as_str();
			//right shift then and is sometimes optimized into rlwinm
			result += format!("{} = ({} >> {}) & {:#X};\n", r_dest, r_source, (32 - shift_amount), bitmask).as_str();
			//result += format!(r_dest + " = (rotl(" + r_source + ", " + shift_amount + ")) & 0x" + NumberToHexString(bitmask) + ";").as_str();
			//for things like (a & 0xFF) << 4, if it gets turned into rlwinm the value to and by gets shifted since it will be applied after (in this case, it becomes (a << 4) & 0xFF0)
			result += format!("{} = ({} & {:#X}) << {};\n", r_dest, r_source, bitmask >> shift_amount, shift_amount).as_str();
		  }
		}

		let range_start = 31 - bitmask_end - shift_amount;
		let range_end = 31 - bitmask_start - shift_amount;
		
		let mut start_bit = range_start as i32;
		let mut end_bit = range_end as i32;
		
		if start_bit < 0 { start_bit += 32; }
		if end_bit < 0 { end_bit += 32; }
		if start_bit > 31 { start_bit %= 31; }
		if end_bit > 31 { end_bit %= 31; }
		
		let bits = (end_bit - start_bit).abs() + 1;
		
		if bits > 1 {
		result += format!("Other info: accesses  bits {}-{}\n", start_bit, end_bit).as_str();
		}else{
		  result += format!("Other info: accesses bit {}\n", start_bit).as_str();
		}
	}else if instruction_type == InstructionType::Rlwimi || instruction_type == InstructionType::Inslwi || instruction_type == InstructionType::Insrwi {
		 //Rlwimi instructions
		//If the destination and source registers are the same, and the shift amount is 0, then add &= (only anding with a given bitmask)
		if r_dest == r_source && shift_amount == 0 {
			result += format!("{} = {};\n", r_dest, r_dest).as_str();
		} else if shift_amount == 0 {
			//r_dest = (r_source & bitmask) | (r_dest && ~bitmask)
			result += format!("{} = ({} & {:#X}) | ({} & {:#X});\n", r_dest, r_source, bitmask, r_dest, !bitmask).as_str();
			result += "Could also be:\n";
			result += format!("{} = ({} & ~{:#X}) | ({} & ~{:#X});\n", r_dest, r_source, !bitmask, r_dest, bitmask).as_str();
		} else {
			//r_dest = ((r_source << shift_amount) & bitmask) | (r_dest & ~bitmask)
			result += format!("{} = (({} << {}) & {:#X}) | ({} & {:#X});\n", r_dest, r_source, shift_amount, bitmask, r_dest, !bitmask).as_str();
			result += "Could also be:\n";
			result += format!("{} = (({} << {} & ~{:#X}) | ({} & ~{:#X});\n", r_dest, r_source, shift_amount, !bitmask, r_dest, bitmask).as_str();
		}
	}else{
		 //Rlwnm instructions
		result += format!("{} = ({} << {}) & {:#X};\n", r_dest, r_source, rshift_amount, bitmask).as_str();
		result += "Could also be:\n";
		result += format!("{} = ({} << {}) & ~{:#X};\n", r_dest, r_source, rshift_amount, !bitmask).as_str();

		let range_start = 31 - bitmask_end - shift_amount;
		let range_end = 31 - bitmask_start - shift_amount;
		
		let mut start_bit = range_start as i32;
		let mut end_bit = range_end as i32;
		
		if start_bit < 0 { start_bit += 32; }
		if end_bit < 0 { end_bit += 32; }
		if start_bit > 31 { start_bit %= 31; }
		if end_bit > 31 { end_bit %= 31; }
		
		let bits = (end_bit - start_bit).abs() + 1;
		
		if bits > 1 {
		result += format!("Other info: accesses  bits {}-{}\n", start_bit, end_bit).as_str();
		}else{
		  result += format!("Other info: accesses bit {}\n", start_bit).as_str();
		}
	}

	if sets_flags {
		result += "Also sets EQ (= 0), GT (> 0), LT (< 0), and SO flags\n";
	}
	
	Some(result)
}

fn generate_bitmask(start_index: u32, end_index: u32) -> u32 {
	let mut bitmask = 0;
	let mut i = start_index;

	loop {
		bitmask |= 1 << (31 - i);
		if i == end_index { break; }
		i += 1;
		if i > 31 { i = 0; }
	}

	bitmask
}

fn check_if_valid_reg_string(s: &str) -> bool {
	if s.len() < 2 || s.len() > 3 || !s.starts_with('r') || (s.contains("r0") && s.len() > 2) {
		return false;
	}

	let result = match s[1..].parse::<i32>() {
		Ok(num) => num,
		Err(_e) => { return false; }
	};
	if !(0..=31).contains(&result) {
		return false;
	}

	true
}
