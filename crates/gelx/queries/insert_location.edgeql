with NewLocation := (insert Location {
	point := <ext::postgis::geometry>$point,
	area := <ext::postgis::geography>$area,
})
select NewLocation {
	point,
	area,
};