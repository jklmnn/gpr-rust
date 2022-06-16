package body Test2 is

   function Add (A : Interfaces.C.int;
                 B : Interfaces.C.int) return Interfaces.C.int
   is
      use type Interfaces.C.int;
   begin
      return A + B;
   end Add;

end Test2;
