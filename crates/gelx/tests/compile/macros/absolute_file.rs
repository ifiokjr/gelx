use gelx_macros::gelx_raw;

fn main() {
	gelx_raw!(insert_user, file: "/absolute/path/to/queries/insert_user.edgeql");
}
