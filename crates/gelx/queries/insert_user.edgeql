with NewUser := (insert User {
  name := <str>$name,
  bio := <str>$bio,
  slug := <str>$slug,
})
select NewUser {
  id,
  name,
  bio,
  slug,
};
