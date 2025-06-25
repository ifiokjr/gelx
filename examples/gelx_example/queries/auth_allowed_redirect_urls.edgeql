# This query is used to set the allowed redirect URLs for the auth system. Unfortunately,
# `configure` can't be used with parameters.
#
# `non-constant expression in CONFIGURE DATABASE SET`
configure current branch set ext::auth::AuthConfig::allowed_redirect_urls := {
    'https://example.com',
    'https://example.com/auth',
    'https://localhost:3000',
    'https://localhost:3000/auth'
};
