
(define fac
    (lambda (n)
        (if (= n 0) 1 (* n (fac (- n 1))))))

(display (fac 4))

