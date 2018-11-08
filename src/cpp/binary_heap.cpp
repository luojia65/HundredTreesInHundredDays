#include <iostream>
#include <cstdlib>
const size_t DEFAULT_CAPACITY = 4;

template<class T>
struct binary_heap {
	T* ptr;
    size_t len, cap;
    binary_heap() {
    	ptr=NULL;
    	this->clear();
	} 
	
	~binary_heap() {
		free(this->ptr);
	} 
    
    void push(T value) {
    	if (this->ptr==NULL||this->len==this->cap) 
    		this->buf_double();
		this->ptr[++this->len]=value;
		size_t now=this->len,nxt;
		while(now>1) {
			nxt=now>>1;
			if(this->ptr[now]<this->ptr[nxt]) 
				this->swap(now, nxt);
			now=nxt;
		}
    }
    
    T pop() {
    	T ans=this->ptr[1];
    	this->swap(1, this->len--);
    	size_t now=1,nxt;
    	while(1) {
			nxt=now<<1;
			if(nxt>this->len) break;
			if((nxt+1)<=this->len && this->ptr[nxt+1]<this->ptr[nxt])
				nxt++;
			if(this->ptr[nxt]<this->ptr[now])
				this->swap(now, nxt);
			now=nxt;
		}
		return ans;
	}
	
	T peek() {
		return this->ptr[this->len];
	}
	
	bool is_empty() {
		return this->len == 0;
	}
	
	void clear() {
		if(this->is_empty()) return;
    	this->len=0;
		this->cap=DEFAULT_CAPACITY;
		if(this->ptr!=NULL) free(this->ptr);
	}
    
    void buf_double() {
		T* new_ptr = (T*)malloc((1+this->cap)*sizeof(T));
		if(this->ptr!=NULL) {
			memcpy(new_ptr, this->ptr, (1+this->len)*sizeof(T));
			free(this->ptr);
			(this->cap)*=2;
		}
		this->ptr = new_ptr;
	}
	
	void swap(size_t a, size_t b) {
		T c=*((this->ptr)+a);
		*((this->ptr)+a)=*((this->ptr)+b);
		*((this->ptr)+b)=c;
	}
};

int main() {
	binary_heap<int> heap;
	char c;int i;
	while(~scanf("%c",&c)) {
		if(c=='i') scanf("%d\n",&i), heap.push(i);
		if(c=='o') 
			if (heap.is_empty()) std::cout<<"pop: empty"<<std::endl;
			else std::cout<<heap.pop()<<std::endl;
		if(c=='c') heap.clear();
		if(c=='p') 
			if (heap.is_empty()) std::cout<<"peak: empty"<<std::endl;
			else std::cout<<heap.peek()<<std::endl;
	}
    return 0;
}
