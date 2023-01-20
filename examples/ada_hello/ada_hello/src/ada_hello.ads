package Ada_Hello
is

   function Hello (A : Integer; B : Integer) return Integer with
      Export,
      Convention => C,
      External_Name => "ada_hello";

end Ada_Hello;
