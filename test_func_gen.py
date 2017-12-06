def f(x,y,z):
    return (x + y + z**2)**3 + y * x

if __name__ == '__main__':
    val_range = []
    iter = 0
    val = 0
    while val < 2147483647:
        val_range.append(val)
        
        if -val != val: val_range.append(-val)

        iter += 1
        val = iter ** 13

    for i in list(val_range):
        for j in list(val_range):
            for k in list(val_range):
                r = f(i,j, k)
                if r < 2147483647 and r > -2147483648:
                    print("vec![{},{},{}] => {},".format(i,j,k,r))

