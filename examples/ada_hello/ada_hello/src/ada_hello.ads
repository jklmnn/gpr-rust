package Ada_Hello is

   function Hello return Integer with
     Export, Convention => C, External_Name => "ada_hello";

end Ada_Hello;
