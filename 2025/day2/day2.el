(require 'ert)

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

(defun multiples-in-range (n range)
  (- (/ (cadr range) n) (/ (1- (car range)) n)))

(ert-deftest test-multiples-in-range ()
  "Test multiples-in-range"
  (should (equal (multiples-in-range 10 '(10 20)) 2))
  (should (equal (multiples-in-range 11 '(10 10)) 0))
  (should (equal (multiples-in-range 11 '(10 23)) 2))
  (should (equal (multiples-in-range 11 '(12 21)) 0)))

(defun next-multiple (f n)
  (* f (/ (+ n (1- f)) f)))

(defun least-multiple-in-range (f range)
  (let ((first (next-multiple f (car range))))
    (when (<= first (cadr range))
      (/ first f))))

(ert-deftest test-least-multiple-in-range ()
  (should (equal (least-multiple-in-range 10 '(9 11)) 1))
  (should (equal (least-multiple-in-range 10 '(29 100)) 3)))

(defun prev-multiple (f n)
  (* f (/ n f)))

(defun greatest-multiple-in-range (f range)
  (let ((last (prev-multiple f (cadr range))))
    (when (<= (car range) last)
      (/ last f))))

(ert-deftest test-greatest-multiple-in-range ()
  (should (equal (greatest-multiple-in-range 10 '(9 11)) 1))
  (should (equal (greatest-multiple-in-range 10 '(29 100)) 10)))

(defun sum-of-range (range)
  (let ((size (1+ (- (cadr range) (car range)))))
    (/ (* size (+ (car range) (cadr range))) 2)))

(ert-deftest test-sum-of-range ()
  (should (equal (sum-of-range '(1 10)) 55))
  (should (equal (sum-of-range '(10 15)) 75)))
  
(defun intersect-ranges (left right)
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

(defun power-dup (power) (1+ (* 10 power)))

(defun power-dup-range (power dup)
  (let ((high (1- (* 10 power))))
    (list (* power dup) (* high dup))))

(ert-deftest test-power-invalid-range ()
  (should (equal (power-dup-range 1 11) '(11 99)))
  (should (equal (power-dup-range 10 101) '(1010 9999)))
  (should (equal (power-dup-range 100 1001) '(100100 999999))))

(defun sum-of-invalid-ids-in-range-for-power (power range)
  (let* ((dup (power-dup power))
         (invalid-range (power-dup-range power dup)))
    (if-let ((range (intersect-ranges invalid-range range))
             (least (least-multiple-in-range dup range))
             (greatest (greatest-multiple-in-range dup range)))
        (* dup (sum-of-range (list least greatest)))
      0)))

(ert-deftest test-sum-of-invalid-ids-in-range-for-power ()
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(1 1)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(10 10)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(11 11)) 11))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(11 22)) 33))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(22 44)) 99))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(22 99)) 484))
  (should (equal (sum-of-invalid-ids-in-range-for-power 1 '(22 1000)) 484))
  (should (equal (sum-of-invalid-ids-in-range-for-power 100 '(1 1)) 0))
  (should (equal (sum-of-invalid-ids-in-range-for-power 100 '(100100 100100)) 100100))
  (should (equal (sum-of-invalid-ids-in-range-for-power 100 '(100100 101101)) 201201))
  (should (equal (sum-of-invalid-ids-in-range-for-power 100 '(100100 200200)) 15165150))
  (should (equal (sum-of-invalid-ids-in-range-for-power 100 '(100000 200299)) 15165150)))

(defun range-fully-before (left right)
  (< (cadr left) (car right)))

(defun sum-of-invalid-ids-for-range (range)
  (let ((power 1)
        (sum 0)
        dup
        dup-range)
    (while (progn
             (setq dup (power-dup power)
                   dup-range (power-dup-range power dup))
             (not (range-fully-before range dup-range)))
      (cl-incf sum (sum-of-invalid-ids-in-range-for-power power range))
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
  (let ((sum 0))
    (while ranges
      (cl-incf sum (sum-of-invalid-ids-for-range (car ranges)))
      (setq ranges (cdr ranges)))
    sum))

(sum-of-invalid-ids part1-test-input) ; 1227775554
(sum-of-invalid-ids part1-input) 40398804950
