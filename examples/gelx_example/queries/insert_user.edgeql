select (insert User {
  name := <str>$name,
  bio := <str>$bio,
  slug := <str>$slug,
}) {
  id,
  name,
  bio,
  slug,
};
