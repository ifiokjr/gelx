select assert_single((
	select TestUser { id, public_id } filter .active and .namelc = str_lower(<str>$username)
))
