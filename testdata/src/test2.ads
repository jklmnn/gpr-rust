with Interfaces.C;

package Test2 is

   function Add (A : Interfaces.C.int;
                 B : Interfaces.C.int) return Interfaces.C.int with
      Export,
      Convention => C,
      External_Name => "test2_add";

end Test2;
