# If KERNELRELEASE is defined, we are inside Kbuild; use its syntax  
ifneq ($(KERNELRELEASE),)  
	# List of object files to include in the module  
	obj-m += lchacha.o  # Declare the module target  
	lchacha-objs := chacha.o chacha_proc.o chacha_ioctl.o chacha_rw.o ../chacha20/chacha20.o  # mymodule.ko depends on these objects  
 
# Otherwise, we are outside Kbuild; invoke Kbuild  
else  
	KERNELDIR ?= /lib/modules/$(shell uname -r)/build  
	PWD := $(shell pwd)  
 
all:  
	$(MAKE) -C $(KERNELDIR) M=$(PWD) modules  
 
clean:  
	$(MAKE) -C $(KERNELDIR) M=$(PWD) clean  
endif  
