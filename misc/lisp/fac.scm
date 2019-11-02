
(define fac
    (lambda n
        (if (eq n 0) 1 (mul n (fac (sub n 1))))))

(display (fac 4))

