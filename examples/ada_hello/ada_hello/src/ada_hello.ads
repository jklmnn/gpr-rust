package Ada_Hello
is

   procedure Hello with
      Export,
      Convention => C,
      External_Name => "ada_hello";

end Ada_Hello;
