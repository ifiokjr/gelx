CREATE MIGRATION m14fgr2cnwdlfq6rejide5ryxfa2y6dn2jlou47s4geadl5powkfma
    ONTO m1umlytq2fg6arigkzjynrhtn5buuspjl4by7vznvvyyf7uw2hfofq
{
  CREATE TYPE default::Location EXTENDING default::CreatedAt, default::UpdatedAt {
      CREATE REQUIRED PROPERTY area: ext::postgis::geography;
      CREATE REQUIRED PROPERTY point: ext::postgis::geometry;
  };
};
