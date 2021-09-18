mod header_size {
	use crate::HeaderSize;

	#[test]
	fn header_size_test() {
		let hs1 = HeaderSize::One;
		let hs2 = HeaderSize::Two;
		let hs5 = HeaderSize::Five;

		assert_eq!(hs1 as u8, 1);
		assert_eq!(hs2 as u8, 2);
		assert_eq!(hs5 as u8, 5);
	}

	#[test]
	fn header_size_to_font_size_conversion_test() {
		assert_eq!(HeaderSize::One.to_font_size(), 90);
		assert_eq!(HeaderSize::Two.to_font_size(), 80);
		assert_eq!(HeaderSize::Three.to_font_size(), 70);
		assert_eq!(HeaderSize::Four.to_font_size(), 60);
		assert_eq!(HeaderSize::Five.to_font_size(), 50);
	}
}
