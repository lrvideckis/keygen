# keygen

An(other) algorithm for generating optimal keyboard layouts.

This code follows the simulated annealing method used in
[Carpalx](http://mkweb.bcgsc.ca/carpalx/?simulated_annealing), but while
optimizing for the best mobile keyboard layout using techniques used by
[MessagEase](https://www.exideas.com/ME/index.php) in [their
paper](https://www.exideas.com/ME/ICMI2003Paper.pdf).

First I note I optimized my layout for one-thumb typing.

Think of one-thumb typing as identical to moving a mouse around an on-screen
keyboard and clicking the keys to type. Under this model, we can use [Fitts's
Law](https://en.wikipedia.org/wiki/Fitts%27s_law) to calculate the time to move
the cursor to each next key and tap it. Now this works well if the keyboard
only had tap-to-type keys, but this layout is optimized for the [Unexpected
Keyboard](https://play.google.com/store/apps/details?id=juloo.keyboard2) where
you also can swipe in a direction to type that key. So how to model swipes?

Inspired by MessagEase's paper: a swipe has a start location (the center of
that key) and an end location (a short distance away in the direction of that
letter). So we model this as 2 clicks + a constant. E.g. 2 applications of
Fitts's law + a constant. The constant can be thought of as a penalty for
swiping.

------

Now here's the main difference between my algorithm and MessagEases:

So the way MessagEase works is:

- the center letter is typed via a double tap

- the edge letters are typed via 2 taps, one in the key, and the second in the adjacent key in the direction of the letter.

Whereas the way the Unexpected Keyboard works is:

- the center letter is typed via a single tap

- the edge letters are typed via a swipe in the direction of the letter

Now looking at MessagEase's layout, the center key has 9 total letters, so when the
interaction is 2-taps, it's fine. But when the interaction is a swipe outwards,
your swipe direction has to be quite precise. So in my mind, having 9 total
letters in a single key should have some penalty.

So I added a penalty when 2 swipe-letters are adjacent. This is pretty much the
main difference between my algorithm and MessagEase's.

--------

```
Reference: MESSAGEASE
       |       |       |
   a   |   n   |   i   |
     v |   l   | x     |
------- ------- -------
       | q u p |       |
   h k | c o b | m r   |
       | g d j |       |
------- ------- -------
     y |   w   | f     |
   t   |   e z |   s   |
       |       |       |
------- ------- -------
       |       |       |
       |   S   |       |
       |       |       |
------- ------- -------

total: 1103140.6725019943; scaled: 0.3647234332206442
base: 212243.0999999959  /  d: 32771.399999999914; l: 27170.099999999842; u: 20160.899999999925; w: 18682.799999999974; m: 18507.29999999992;
swipe penalty: 890897.5725019489  /  d : 35473.193569306175;  a: 25813.315814681046;  t: 19310.48316359531; e : 18788.775510204076;  i: 17164.790641031486;

Reference: THUMB KEY
       |       |       |
   s   |   r   |   o   |
     w |   g   | u     |
------- ------- -------
       | j q b |       |
   n m | k h p | l a   |
       | v x y |       |
------- ------- -------
     c |   f   | d     |
   t   |   i z |   e   |
       |       |       |
------- ------- -------
       |       |       |
       |   S   |       |
       |       |       |
------- ------- -------

total: 1101759.2294231656; scaled: 0.3642666966728324
base: 212243.0999999959  /  d: 32771.399999999914; l: 27170.099999999842; u: 20160.899999999925; w: 18682.799999999974; m: 18507.29999999992;
swipe penalty: 889516.1294231324  /  d : 27169.015047194454; e : 23890.929562399564;  a: 21257.203797570757;  t: 19310.48316359531; s : 18413.49021703077;

Reference: best layout I found, keeping letter positions identical to MessagEase/Thumb Key
       |       |       |
   n   |   i   |   r   |
     g |   d   | m     |
------- ------- -------
       | v z q |       |
   a l | x e p | u o   |
       | k y j |       |
------- ------- -------
     c |   b   | w     |
   s   |   t f |   h   |
       |       |       |
------- ------- -------
       |       |       |
       |   S   |       |
       |       |       |
------- ------- -------

total: 1027231.230500423; scaled: 0.33962604266039687
base: 212243.0999999959  /  d: 32771.399999999914; l: 27170.099999999842; u: 20160.899999999925; w: 18682.799999999974; m: 18507.29999999992;
swipe penalty: 814988.130500393  /  e : 29779.504618141487; d : 25718.750659119174;  a: 21257.203797570757; he: 16805.523156363;  i: 16686.12244897961;

Reference: my layout, without symbols
       |       |       |
   r z |   i x |   n   |
   q   |   l   | g     |
------- ------- -------
     u |   k   | v     |
   o   | p e   |   a   |
     f |   d   |       |
------- ------- -------
 j   m | b   w | c     |
   s   |   t   |   h   |
       |       | y     |
------- ------- -------
       |       |       |
       |   S   |       |
       |       |       |
------- ------- -------

total: 983122.6793116957; scaled: 0.32504275095068785
base: 212243.09999999363  /  d: 32771.399999999914; l: 27170.099999999817; u: 20160.89999999994; w: 18682.799999999952; m: 18507.299999999916;
swipe penalty: 770879.5793116714  /  e : 29779.50461814148;  a: 21257.20379757075; d : 17188.51865619149; he: 16805.523156363004;  i: 16686.122448979604;

Reference: my layout with symbols
       |       |       |
   r z |   i x |   n   |
   q   |   l   | g     |
------- ------- -------
     u |   k   | v     |
   o   | p e - |   a   |
 ?   f |   d   | '   ! |
------- ------- -------
 j   m | b   w | c     |
   s   |   t   |   h   |
     . |   ,   | y     |
------- ------- -------
       |       |       |
       |   S   |       |
       |       |       |
------- ------- -------

total: 1064955.6507521062; scaled: 0.3520985952671701
base: 242207.69999998302  /  d: 32771.399999999805; l: 27170.09999999979; u: 20160.899999999907; w: 18682.79999999993; m: 18507.299999999905;
swipe penalty: 822747.9507520351  /  e : 29779.504618141495;  a: 21257.20379757074; d : 17188.5186561915; he: 16805.523156362993;  i: 16686.122448979597;
```

## Installing and running

You'll need a recent-ish version of [Rust](https://www.rust-lang.org/).

Then: `cargo run -- run corpus/books.short.txt`.

## Installing the (upcoming) optimal keyboard layout

If you're crazy enough to want to try this, you're probably smart enough to figure out how to install custom keyboards on your system of choice.

## Credits

The simulated annealing algorithm and corpus are taken from Carpalx by Martin Krzywinski.

## Other alternate keyboard layouts

mdickens has a good list of them [here](http://mdickens.me/typing/alternative_layouts.html).

## Licence

MIT
