(require 'ert)
(require 'dash)

(defconst part1-test-input '(
(11 22)
(95 115)
(998 1012)
(1188511880 1188511890)
(222220 222224)
(1698522 1698528)
(446443 446449)
(38593856 38593862)
(565653 565659)
(824824821 824824827)
(2121212118 2121212124)
))

;;;; A range is a list of two integers `(start end)', representing all
;;;; integers `i' such that `start <= i <= end'.

(defun multiples-in-range (n range)
  "The number of multiples of `n' that occur in `range'."
  (- (/ (cadr range) n) (/ (1- (car range)) n)))

(ert-deftest test-multiples-in-range ()
  "Test multiples-in-range"
  (should (equal (multiples-in-range 10 '(10 20)) 2))
  (should (equal (multiples-in-range 11 '(10 10)) 0))
  (should (equal (multiples-in-range 11 '(10 23)) 2))
  (should (equal (multiples-in-range 11 '(12 21)) 0)))

(defun next-multiple (f n)
  "The least multiple of `f' greater than or equal to `n'."
  (* f (/ (+ n (1- f)) f)))

(defun least-multiple-in-range (f range)
  "The smallest multiple of `f` that falls within `range', or `nil'."
  (let ((first (next-multiple f (car range))))
    (when (<= first (cadr range))
      (/ first f))))

(ert-deftest test-least-multiple-in-range ()
  (should (equal (least-multiple-in-range 10 '(9 11)) 1))
  (should (equal (least-multiple-in-range 10 '(29 100)) 3)))

(defun prev-multiple (f n)
  "The largest multiple of `f' less than or equal to `n'."
  (* f (/ n f)))

(defun greatest-multiple-in-range (f range)
  "The largest multiple of `f` that falls within `range', or `nil'."
  (let ((last (prev-multiple f (cadr range))))
    (when (<= (car range) last)
      (/ last f))))

(ert-deftest test-greatest-multiple-in-range ()
  (should (equal (greatest-multiple-in-range 10 '(9 11)) 1))
  (should (equal (greatest-multiple-in-range 10 '(29 100)) 10)))

(defun sum-of-range (range)
  "The sum of all integers in `range'."
  (-let* (((start end) range)
          (size (1+ (- end start))))
    (/ (* size (+ start end)) 2)))

(ert-deftest test-sum-of-range ()
  (should (equal (sum-of-range '(1 10)) 55))
  (should (equal (sum-of-range '(10 15)) 75)))
  
(defun intersect-ranges (left right)
  "The intersection of the two ranges `left' and `right'.
Return `nil` if the intersection is empty."
  (when (and (>= (cadr left) (car right))
             (>= (cadr right) (car left)))
    (list (max (car left) (car right))
          (min (cadr left) (cadr right)))))

(ert-deftest test-intersect-ranges ()
  (should (equal (intersect-ranges '(1 2) '(3 4)) nil))
  (should (equal (intersect-ranges '(1 2) '(2 4)) '(2 2)))
  (should (equal (intersect-ranges '(1 4) '(2 3)) '(2 3)))
  (should (equal (intersect-ranges '(2 3) '(1 4)) '(2 3)))
  (should (equal (intersect-ranges '(1 3) '(2 4)) '(2 3))))

(defun power-dup (power ones)
  "A number containing `1' digits evenly spaced with zeros.
The `ones' argument gives the number of `1' digits the result should
contain, and `power' is a power of ten that establishes the spacing
between the `1' digits.

For example, `(power-dup 1000 4)' is `1001001001'.

These values are called 'dup' values, since you can multiply any digit
sequence you want by a dup to get a certain number of repetitions of
that digit sequence:

(* (power-dup 1000 4) 123)
-> 123123123123"
  (cl-do ((ones ones (1- ones))
          (dup 0 (1+ (* dup power))))
      ((<= ones 0) dup)))

(ert-deftest test-power-dup ()
  (should (equal (power-dup 10 1) 1))
  (should (equal (power-dup 10 2) 11))
  (should (equal (power-dup 10 3) 111))
  (should (equal (power-dup 100 1) 1))
  (should (equal (power-dup 100 2) 101))
  (should (equal (power-dup 100 3) 10101)))
  
(defun power-dup-range (power dup)
  "The range of numbers that can be produced neatly using `dup'.
Given `dup', which was constructed using `power', return
the range of numbers with repeating sequences of digits that
can be constructed using `dup'.

For example, given a `dup' of `1001', we can multiply that by any number
from `100' to `999' to get a six-digit number made from two repetitions
of the multiplier. Multipliers less than `100' won't produce a six-digit number,
and multipliers greater than `999' will carry, breaking the pattern."
  (let ((low (/ power 10))
        (high (1- power)))
    (list (* low dup) (* high dup))))

(ert-deftest test-power-invalid-range ()
  (should (equal (power-dup-range 10 11) '(11 99)))
  (should (equal (power-dup-range 100 101) '(1010 9999)))
  (should (equal (power-dup-range 1000 1001) '(100100 999999))))

(defun sum-of-invalid-ids-in-range-for-power (power dup range)
  "The sum of all invalid two-repetition ids in `range` that can be produced by `dup'.
This assumes that `dup` has only two `1' digits.
The `power' argument must be the power used to construct `dup'."
  (let* ((invalid-range (power-dup-range power dup)))
    (if-let ((range (intersect-ranges invalid-range range))
             (least (least-multiple-in-range dup range))
             (greatest (greatest-multiple-in-range dup range)))
        (* dup (sum-of-range (list least greatest)))
      0)))

(ert-deftest test-sum-of-invalid-ids-in-range-for-power ()
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(1 1)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(10 10)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(11 11)) 11))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(11 22)) 33))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(22 44)) 99))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(22 99)) 484))
  (should (equal (sum-of-invalid-ids-in-range-for-power 10 11 '(22 1000)) 484))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1000 1001 '(1 1)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1000 1001 '(100100 100100)) 100100))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1000 1001 '(100100 101101)) 201201))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1000 1001 '(100100 200200)) 15165150))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1000 1001 '(100000 200299)) 15165150)))

(defun range-fully-before (left right)
  "True if the range `left' falls fully before the range `right'."
  (< (cadr left) (car right)))

(defun sum-of-invalid-ids-for-range (range)
  "The sum of all invalid product ids in `range`."
  (let ((sum 0)
        (power 10)
        dup
        dup-range)
    (while (progn
             (setq dup (power-dup power 2)
                   dup-range (power-dup-range power dup))
             (not (range-fully-before range dup-range)))
      (cl-incf sum (sum-of-invalid-ids-in-range-for-power power dup range))
      (setq power (* 10 power)))
    sum))

(ert-deftest test-sum-of-invalid-ids-for-range ()
  (should (equal (sum-of-invalid-ids-for-range '(11 22)) 33))
  (should (equal (sum-of-invalid-ids-for-range '(95 115)) 99))
  (should (equal (sum-of-invalid-ids-for-range '(998 1012)) 1010))
  (should (equal (sum-of-invalid-ids-for-range '(1188511880 1188511890)) 1188511885))
  (should (equal (sum-of-invalid-ids-for-range '(222220 222224)) 222222))
  (should (equal (sum-of-invalid-ids-for-range '(1698522 1698528)) 0))
  (should (equal (sum-of-invalid-ids-for-range '(446443 446449)) 446446))
  (should (equal (sum-of-invalid-ids-for-range '(38593856 38593862)) 38593859))
  (should (equal (sum-of-invalid-ids-for-range '(565653 565659)) 0))
  (should (equal (sum-of-invalid-ids-for-range '(824824821 824824827)) 0))
  (should (equal (sum-of-invalid-ids-for-range '(2121212118 2121212124)) 0)))


(defun sum-of-invalid-ids (ranges)
  "The solution to Day 2, Part 1."
  (let ((sum 0))
    (while ranges
      (cl-incf sum (sum-of-invalid-ids-for-range (car ranges)))
      (setq ranges (cdr ranges)))
    sum))

(ert-deftest test-part1 ()
  (should (equal (sum-of-invalid-ids part1-test-input) 1227775554))
  (should (equal (sum-of-invalid-ids part1-input) 40398804950)))

(defun enumerate-inclusive (first last)
  "A list containing the numbers from `first' to `last', inclusive."
  (when (>= last first)
    (-iota (1+ (- last first)) first)))

(defun multiple-of-p (big little)
  "True if `big' is a multiple of `little'."
  (zerop (mod big little)))

(defun strict-divisors (n)
  "A list of all positive integers less than `n' that divide it."
  (--filter (multiple-of-p n it) (enumerate-inclusive 1 (/ n 2))))

(defun sum-for-group-size (range total-digits group-size)
  "The number of invalid ids in `range' whose length is `total-digits'
whose repeating component is `group-size' digits long."
  (if (multiple-of-p total-digits group-size)
      (let* ((num-groups (/ total-digits group-size))
             (power (expt 10 group-size))
             (dup (power-dup power num-groups))
             (dup-range (power-dup-range power dup)))
        (sum-of-invalid-ids-in-range-for-power power dup range))
      0))

(ert-deftest test-sum-for-group-size ()
  (should (equal (sum-for-group-size '(11 22) 2 1) 33))
  (should (equal (sum-for-group-size '(95 115) 2 1) 99))
  (should (equal (sum-for-group-size '(95 115) 3 1) 111))
  (should (equal (sum-for-group-size '(95 115) 3 2) 0)))

(defun sums-for-group-sizes (range total-digits)
  "A list whose i'th element is the number of invalid ids in `range` with a group size of i+1 digits.
That is, element zero is the sum for one-digit groups, element one the
sum for two-digit groups, and so on."
  (-map (-partial #'sum-for-group-size range total-digits)
        (-iota (/ total-digits 2) 1)))

(ert-deftest test-sums-for-group-sizes ()
  (should (equal (sums-for-group-sizes '(11 22) 2) '(33)))
  (should (equal (sums-for-group-sizes '(95 115) 2) '(99)))
  (should (equal (sums-for-group-sizes '(95 115) 3) '(111)))
  (should (equal (sums-for-group-sizes '(998 1012) 2) '(0)))
  (should (equal (sums-for-group-sizes '(998 1012) 3) '(999)))
  (should (equal (sums-for-group-sizes '(998 1012) 4) '(0 1010)))

  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 2) '(0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 3) '(0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 4) '(0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 5) '(0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 6) '(0 0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 7) '(0 0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 8) '(0 0 0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 9) '(0 0 0 0)))
  (should (equal (sums-for-group-sizes '(1188511880 1188511890) 10) '(0 0 0 0 1188511885)))

  (should (equal (sums-for-group-sizes '(222220 222224) 6) '(222222 222222 222222)))

  (should (equal (sums-for-group-sizes '(1698522 1698528) 8) '(0 0 0 0)))

  (should (equal (sums-for-group-sizes '(446443 446449) 6) '(0 0 446446)))
  (should (equal (sums-for-group-sizes '(38593856 38593862) 8) '(0 0 0 38593859)))
  (should (equal (sums-for-group-sizes '(565653 565659) 6) '(0 565656 0)))
  (should (equal (sums-for-group-sizes '(824824821 824824827) 9) '(0 0 824824824 0)))
  (should (equal (sums-for-group-sizes '(2121212118 2121212124) 10) '(0 2121212121 0 0 0))))

(defun sum-omitting-factors-of-nonzeros (nums)
  "Sum a list of numbers, but for every non-zero x_i, subtract all x_j where j strictly divides i.
Treat the first element of the list as having index 1.

For example, given (5 10 0 40):

- Element 4 is 40. 4 is divisible by 2 and 1, so contribute (- 40 10 5) = 25 to the sum.
- Element 3 is 0. 3 is divisible by 1, but since the value is zero, don't worry about
  its divisor elements.
- Element 2 is 10. 2 is divisible by 1, so contribute (- 10 5) = 5 to the sum.
- Element 1 is 5, which has no strict divisors, so contribute 5 to the sum.

Thus the sum omitting factors of this list would be (+ 25 0 5 5) = 35."
  (-sum
   (-map-indexed (lambda (index n)
                   (if (zerop n) 0
                     (let* ((factors (strict-divisors (1+ index)))
                            (values-at-factors (--map (nth (1- it) nums) factors)))
                       (- n (-sum values-at-factors)))))
                 nums)))

(ert-deftest test-sum-omitting-factors-of-nonzeros ()
  (should (equal (sum-omitting-factors-of-nonzeros '(5 10 20 40)) 50))
  (should (equal (sum-omitting-factors-of-nonzeros '(5 10 0 40)) 35))
  (should (equal (sum-omitting-factors-of-nonzeros '(0 2121212121 0 0 0)) 2121212121)))

(defun distinct-sum-for-range (range total-digits)
  "The sum of the distinct invalid ids of length `total-digits' in `range'."
  (sum-omitting-factors-of-nonzeros (sums-for-group-sizes range total-digits)))

(ert-deftest test-distinct-sum-for-range ()
  (should (equal (distinct-sum-for-range '(11 22) 2) 33))
  (should (equal (distinct-sum-for-range '(95 115) 2) 99))
  (should (equal (distinct-sum-for-range '(95 115) 3) 111))
  (should (equal (distinct-sum-for-range '(998 1012) 3) 999))
  (should (equal (distinct-sum-for-range '(998 1012) 4) 1010))
  (should (equal (distinct-sum-for-range '(1188511880 1188511890) 10) 1188511885))
  (should (equal (distinct-sum-for-range '(222220 222224) 6) 222222))
  (should (equal (distinct-sum-for-range '(1698522 1698528) 7) 0))
  (should (equal (distinct-sum-for-range '(446443 446449) 6) 446446))
  (should (equal (distinct-sum-for-range '(38593856 38593862) 8) 38593859))
  (should (equal (distinct-sum-for-range '(565653 565659) 6) 565656))
  (should (equal (distinct-sum-for-range '(824824821 824824827) 9) 824824824))
  (should (equal (distinct-sum-for-range '(2121212118 2121212124) 10) 2121212121)))

(defun count-digits (n)
  "The length of `n' in digits."
  (cl-do ((digits 1 (1+ digits))
          (pow 10 (* pow 10)))
      ((> pow n) digits)))

(defun part2-sum-of-invalid-ids-for-range (range)
  "The sum of the distinct invalid ids in `range'."
  (let* ((min-digits (count-digits (car range)))
         (max-digits (count-digits (cadr range)))
         (lengths (enumerate-inclusive min-digits max-digits)))
    (-sum (--map (distinct-sum-for-range range it) lengths))))
    
(ert-deftest test-part2-sum-of-invalid-ids-for-range ()
  (should (equal (part2-sum-of-invalid-ids-for-range '(11 22)) 33))
  (should (equal (part2-sum-of-invalid-ids-for-range '(95 115)) 210))
  (should (equal (part2-sum-of-invalid-ids-for-range '(998 1012)) 2009))
  (should (equal (part2-sum-of-invalid-ids-for-range '(1188511880 1188511890)) 1188511885))
  (should (equal (part2-sum-of-invalid-ids-for-range '(222220 222224)) 222222))
  (should (equal (part2-sum-of-invalid-ids-for-range '(1698522 1698528)) 0))
  (should (equal (part2-sum-of-invalid-ids-for-range '(446443 446449)) 446446))
  (should (equal (part2-sum-of-invalid-ids-for-range '(38593856 38593862)) 38593859))
  (should (equal (part2-sum-of-invalid-ids-for-range '(565653 565659)) 565656))
  (should (equal (part2-sum-of-invalid-ids-for-range '(824824821 824824827)) 824824824))
  (should (equal (part2-sum-of-invalid-ids-for-range '(2121212118 2121212124)) 2121212121)))

(defun part2-sum-of-invalid-ids (ranges)
  "Day 2 Part 2."
  (-sum (-map #'part2-sum-of-invalid-ids-for-range ranges)))

(ert-deftest test-part2 ()
  (should (equal (part2-sum-of-invalid-ids part1-test-input) 4174379265))
  (should (equal (part2-sum-of-invalid-ids part1-input) 65794984339)))
