with Ada.Text_IO;

package body Ada_Hello is

   function Hello return Integer is
      A : Integer := 7;
      B : Integer := 9;
   begin
      -- Ada.Text_IO.Put_Line (Integer'Image(A+B));
      return (A + B);
   end Hello;

end Ada_Hello;
