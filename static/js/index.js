class Auth extends HTMLElement {
    constructor() {
        super();

        this.attachShadow({ mode: 'open' });
        this._step = 'login'


        if(this._getJwt()){
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

       document.addEventListener('_auth_login', this._login.bind(this));
       document.addEventListener('_auth_signup', this._signup.bind(this));
       document.addEventListener('_auth_logout', this._logout.bind(this));

    }

    disconnectedCallback() {
       document.removeEventListener('_auth_login', this._login);
       document.removeEventListener('_auth_signup', this._signup);
       document.removeEventListener('_auth_logout', this._logout);
    }

    _render(){
        if( this._step == 'login'){
            this.shadowRoot.innerHTML = `<hf-auth-login></hf-auth-login>`
        }
        else if (this._step == 'logout'){
            this.shadowRoot.innerHTML = `<hf-auth-logout></hf-auth-logout>`
        }
        else if (this._step == 'signup'){
            this.shadowRoot.innerHTML = `<hf-auth-signup passphrase="${this._passphrase}"></hf-auth-signup>`
        }
        else{
            console.error({service:'auth', error: 'unknown state :' +this._step})
            this.shadowRoot.innerHTML = ''
        }
    }

    _getJwt(){

        // TODO add refresh
        return localStorage.getItem("jwt")
    }

    _setJwt(jwt){
        localStorage.setItem("jwt", jwt)

        var event = new CustomEvent('auth-jwt', { 'detail': jwt });
        document.dispatchEvent(event);
    }


    _login(e) {
        this._step = 'logout'
        this._setJwt(e.detail)
        this._render()
    }

    _signup(e) {
        this._passphrase = e.detail
        this._step = 'signup'
        this._render()
    }

    _logout(e) {
        this._step = 'login'
        localStorage.removeItem("jwt")
        this._render()
    }
}


customElements.define('hf-auth', Auth);