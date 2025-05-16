use gelx_macros::gelx_raw;

fn main() {
	gelx_raw!(example, query: "select bool", "more stuff", that is, "not allowed");
}
