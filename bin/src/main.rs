fn main(){
	if let Some(output) = rlwinmdec::decode("rotlw r3, r4, r5") {
		println!("{}", output);
	}else{
		println!("Something sus happened");
	}
}
