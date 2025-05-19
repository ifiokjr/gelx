select User {
	id,
  name,
  bio,
  slug,
} filter .slug = <str>$slug;