;name stone
org   start
start mov.i <2, 3
      add.f d1, start
      jmp   start
      dat   #0
d1    dat   #-4, #4
end 
