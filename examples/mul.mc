[multiplication]
s:~ 69 [set a multiplicand to slot 0]
s:~ 420 [set a multiplier to slot 1]

s:~ 0 [initialize slot 2]
;repeat [set a jump label]
s:2 a:.2 .0 [add value of slot 2 with slot 0 and then set it to slot 2]
s:1 a:.1 -1 [decrement the value of slot 1]
?:.1 j:"repeat" [if value of slot 1 is non zero then jump to repeat]

p:.2 [print the shit]
