class Auth extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this._step = 'login'


        if(localStorage.getItem("jwt")){
            this._step = 'logout'
        }else{
            this._step = 'login'
        }

        this._render()
    }

    connectedCallback() {
        import('http://localhost:8000/js/login.js').then(module => {
            console.log('login loaded')
        });

        import('http://localhost:8000/js/logout.js').then(module => {
            console.log('logout loaded')
        });

        import('http://localhost:8000/js/signup.js').then(module => {
            console.log('signup loaded')
        });

       document.addEventListener('logout', this._logout.bind(this));

    }

    disconnectedCallback() {
       document.removeEventListener('logout', this._logout);
    }

    _render(){
        if( this._step == 'login'){
            this.shadowRoot.innerHTML = `<hf-auth-login></hf-auth-login>`
        }
        else if (this._step == 'logout'){
            this.shadowRoot.innerHTML = `<hf-auth-logout></hf-auth-logout>`
        }
        else if (this._step == 'signup'){
            this.shadowRoot.innerHTML = `<hf-auth-signup></hf-auth-signup>`
        }
        else{
            console.error({service:'auth', error: 'unknown state :' +this._step})
            this.shadowRoot.innerHTML = ''
        }

    }

    _getJwt(){

        // TODO add refresh
        return localStorage.getItem("data")
    }

    _setJwt(jwt){
        localStorage.setItem("jwt", jwt)

        var event = new CustomEvent('login', { 'detail': jwt });
        document.dispatchEvent(event);
    }

    _logout(e) {
        e.preventDefault();
        console.log('receive event')

        localStorage.removeItem("jwt")
    }
}


customElements.define('hf-auth', Auth);