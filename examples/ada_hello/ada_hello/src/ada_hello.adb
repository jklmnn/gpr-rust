with Ada.Text_IO;

package body Ada_Hello is

   function Hello (A : Integer; B : Integer) return Integer
   is
   begin
      Ada.Text_IO.Put_Line ("Hello from Ada! blubb");
      return A + B;
   end Hello;

end Ada_Hello;
