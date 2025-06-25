CREATE MIGRATION m1sxwa4zhaq4adwbh6ansh4kjh4iuqdy77shmazpjhspslcmvlq6oa
    ONTO initial
{
  CREATE EXTENSION pgcrypto VERSION '1.3';
  CREATE EXTENSION auth VERSION '1.0';
  CREATE EXTENSION postgis VERSION '3.5';
  CREATE MODULE additional IF NOT EXISTS;
  CREATE SCALAR TYPE additional::Awesomeness EXTENDING enum<Very, Somewhat, NotReally>;
  CREATE SCALAR TYPE additional::smartness EXTENDING enum<low, mid, genius>;
  CREATE SCALAR TYPE default::AccountProvider EXTENDING enum<Github>;
  CREATE SCALAR TYPE default::Position EXTENDING std::int32 {
      CREATE CONSTRAINT std::max_value(1_000_000);
      CREATE CONSTRAINT std::min_value(0);
  };
  CREATE SCALAR TYPE default::RelationshipType EXTENDING enum<Follow, Block, Mute>;
  CREATE SCALAR TYPE default::Role EXTENDING enum<None, Editor, Moderator, Admin, Owner>;
  CREATE SCALAR TYPE default::UserId EXTENDING std::sequence;
  CREATE GLOBAL additional::alternative -> std::str;
  CREATE GLOBAL default::current_user_id -> std::uuid;
  CREATE ABSTRACT TYPE default::CreatedAt {
      CREATE REQUIRED PROPERTY created_at: std::datetime {
          SET default := (std::datetime_of_statement());
          SET readonly := true;
      };
  };
  CREATE ABSTRACT TYPE default::UpdatedAt {
      CREATE REQUIRED PROPERTY updated_at: std::datetime {
          SET default := (std::datetime_of_statement());
          CREATE REWRITE
              INSERT 
              USING (std::datetime_of_statement());
          CREATE REWRITE
              UPDATE 
              USING (std::datetime_of_statement());
      };
  };
  CREATE ABSTRACT TYPE default::RelationshipTarget {
      CREATE ANNOTATION std::description := 'A potential target that can be followed, blocked and muted';
  };
  CREATE ABSTRACT TYPE default::Actor {
      CREATE REQUIRED PROPERTY slug: std::str {
          CREATE CONSTRAINT std::exclusive;
          CREATE CONSTRAINT std::regexp('^(?=.{3,39}$)(?![_.-])(?!.*[_.-]{2})[a-zA-Z0-9._-]+(?<![_.-])$');
      };
  };
  CREATE TYPE default::User EXTENDING default::Actor, default::CreatedAt, default::RelationshipTarget, default::UpdatedAt {
      CREATE PROPERTY bio: std::str;
      CREATE PROPERTY name: std::str;
  };
  CREATE TYPE default::Account EXTENDING default::CreatedAt, default::UpdatedAt {
      CREATE REQUIRED LINK user: default::User {
          ON TARGET DELETE DELETE SOURCE;
      };
      CREATE PROPERTY access_token: std::str;
      CREATE PROPERTY access_token_expires_at: std::datetime;
      CREATE REQUIRED PROPERTY provider: default::AccountProvider;
      CREATE REQUIRED PROPERTY provider_account_id: std::str;
      CREATE PROPERTY refresh_token: std::str;
      CREATE PROPERTY refresh_token_expires_at: std::datetime;
      CREATE PROPERTY scope: std::str;
      CREATE PROPERTY username: std::str;
  };
  CREATE TYPE default::Wallet EXTENDING default::CreatedAt, default::UpdatedAt {
      CREATE REQUIRED LINK actor: default::Actor {
          ON TARGET DELETE DELETE SOURCE;
      };
      CREATE REQUIRED PROPERTY primary: std::bool {
          SET default := false;
      };
      CREATE CONSTRAINT std::exclusive ON ((.actor, .primary)) EXCEPT (NOT (.primary));
      CREATE PROPERTY description: std::str;
      CREATE PROPERTY name: std::str;
      CREATE REQUIRED PROPERTY pubkey: std::str {
          CREATE CONSTRAINT std::exclusive;
      };
  };
  ALTER TYPE default::Actor {
      CREATE MULTI LINK wallets := (.<actor[IS default::Wallet]);
  };
  ALTER TYPE default::User {
      CREATE LINK accounts := (.<user[IS default::Account]);
  };
  CREATE GLOBAL default::current_user := (SELECT
      default::User
  FILTER
      (.id = GLOBAL default::current_user_id)
  );
  CREATE TYPE default::Team EXTENDING default::Actor, default::CreatedAt, default::RelationshipTarget, default::UpdatedAt {
      CREATE ANNOTATION std::description := 'Teams are a collection of users that can create projects in easier ways for groups and organisations.';
      CREATE PROPERTY description: std::str {
          CREATE ANNOTATION std::description := 'The description for the team.';
      };
      CREATE REQUIRED PROPERTY name: std::str {
          CREATE ANNOTATION std::description := 'The name of the team which is required';
      };
  };
  CREATE TYPE default::Project EXTENDING default::CreatedAt, default::RelationshipTarget, default::UpdatedAt {
      CREATE REQUIRED LINK creator: default::Actor {
          ON TARGET DELETE RESTRICT;
      };
      CREATE REQUIRED PROPERTY slug: std::str {
          CREATE CONSTRAINT std::regexp('^(?=.{3,39}$)(?![_.-])(?!.*[_.-]{2})[a-zA-Z0-9._-]+(?<![_.-])$');
      };
      CREATE CONSTRAINT std::exclusive ON ((.creator, .slug));
      CREATE REQUIRED PROPERTY name: std::str;
  };
  CREATE TYPE default::Relationship EXTENDING default::CreatedAt {
      CREATE REQUIRED LINK actor: default::Actor {
          SET readonly := true;
      };
      CREATE REQUIRED LINK target: default::RelationshipTarget {
          SET readonly := true;
      };
      CREATE REQUIRED PROPERTY relationship_type: default::RelationshipType {
          SET readonly := true;
      };
      CREATE CONSTRAINT std::exclusive ON ((.actor, .target, .relationship_type));
  };
  CREATE ABSTRACT TYPE default::VerifiedAt {
      CREATE PROPERTY verified_at: std::datetime;
      CREATE PROPERTY verified := (EXISTS (.verified_at));
  };
  CREATE TYPE default::Email EXTENDING default::VerifiedAt, default::CreatedAt, default::UpdatedAt {
      CREATE REQUIRED LINK user: default::User {
          ON TARGET DELETE DELETE SOURCE;
      };
      CREATE REQUIRED PROPERTY primary: std::bool {
          SET default := false;
      };
      CREATE CONSTRAINT std::exclusive ON ((.user, .primary)) EXCEPT (NOT (.primary));
      CREATE REQUIRED PROPERTY email: std::str {
          SET readonly := true;
          CREATE CONSTRAINT std::exclusive;
          CREATE REWRITE
              INSERT 
              USING (std::str_trim(std::str_lower(.email)));
      };
  };
  CREATE TYPE default::Location EXTENDING default::CreatedAt, default::UpdatedAt {
      CREATE REQUIRED PROPERTY area: ext::postgis::geography;
      CREATE REQUIRED PROPERTY point: ext::postgis::geometry;
  };
  ALTER TYPE default::User {
      CREATE LINK emails := (.<user[IS default::Email]);
      CREATE LINK email := (SELECT
          std::assert_single((SELECT
              .emails
          FILTER
              (.primary = true)
          ))
      );
  };
  CREATE TYPE default::Simple {
      CREATE REQUIRED PROPERTY position: default::Position;
  };
};
