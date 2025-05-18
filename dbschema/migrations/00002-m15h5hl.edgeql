CREATE MIGRATION m15h5hlf7dhzkpyocyiqflw3lhhq6zadpunsbr3iihqurwsxxvqw7q
    ONTO m1k7o3tcbkvar3vkfeigwnorqbwahoxfrxrcfmqbte2tg7dhkd3zea
{
  CREATE MODULE additional IF NOT EXISTS;
  CREATE SCALAR TYPE additional::Awesomeness EXTENDING enum<Very, Somewhat, NotReally>;
  CREATE SCALAR TYPE additional::smartness EXTENDING enum<low, mid, genius>;
};
